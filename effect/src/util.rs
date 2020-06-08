use self::rand::{RngCore, XorShiftRng};
use core::f32::consts::PI;
use std::error::Error;

struct LPFilter {
    prev_output: f32,
    alpha: f32,
}

impl LPFilter {
    fn new(freq: f32) -> Self {
        let dt = 1.0 / SAMPLE_RATE as f32;
        let rcl = 1.0 / (2.0 * PI as f32 * freq);
        LPFilter {
            prev_output: 0.0,
            alpha: dt / (rcl + dt),
        }
    }

    fn add_sample(&mut self, sample: f32) -> f32 {
        self.prev_output += (sample - self.prev_output) * self.alpha;
        self.prev_output
    }
}

struct HPFilter {
    prev_sample: f32,
    prev_output: f32,
    beta: f32,
}

impl HPFilter {
    fn new(freq: f32) -> Self {
        let dt = 1.0 / SAMPLE_RATE as f32;
        let rch = 1.0 / (2.0 * PI as f32 * freq);
        HPFilter {
            prev_sample: 0.0,
            prev_output: 0.0,
            beta: rch / (rch + dt),
        }
    }

    fn add_sample(&mut self, sample: f32) -> f32 {
        let ds = sample - self.prev_sample;
        self.prev_sample = sample;
        self.prev_output = (self.prev_output + ds) * self.beta;
        self.prev_output
    }
}

struct AllPass {
    delay: DelayLine,
    gain: f32,
    prev: f32,
}

impl AllPass {
    pub fn new(
        delay_ms: u64,
        gain: f32,
        mod_freq: f32,
        high_pass: f32,
        low_pass: f32,
        heads: u32,
    ) -> Self {
        AllPass {
            delay: DelayLine::new(delay_ms, mod_freq, high_pass, low_pass, heads),
            gain,
            prev: 0.0,
        }
    }
    pub fn add_sample(&mut self, sample: f32) -> f32 {
        self.delay.add_sample(sample - self.prev * self.gain);
        self.prev = self.delay.get_sample();
        self.prev
    }
}

pub fn sample_to_f32(sample: i16) -> f32 {
    sample as f32 / std::i16::MAX as f32
}

pub fn f32_to_sample(f: f32) -> i16 {
    (f * std::i16::MAX as f32) as i16
}

struct Osc {
    pub sin: f32,
    pub cos: f32,
    freq: f32,
}

impl Osc {
    pub fn new(freq: f32) -> Self {
        Osc {
            sin: (freq * 300000.0).sin(),
            cos: (freq * 300000.0).cos(),
            freq,
        }
    }

    pub fn inc(&mut self) {
        let dt = self.freq / (SAMPLE_RATE as f32);
        self.sin += self.cos * dt;
        self.cos -= self.sin * dt;
    }
}

struct DelayLine {
    mem: Vec<f32>,
    current: usize,
    delay: f32,
    alpha: f32,
    beta: f32,
    lp: f32,
    hp: f32,
    osc: Osc,
    heads: u32,
}

impl DelayLine {
    fn new(delay_ms: u64, mod_freq: f32, high_pass: f32, low_pass: f32, heads: u32) -> Self {
        let dt = 1.0 / SAMPLE_RATE as f32;
        let rcl = 1.0 / (2.0 * PI as f32 * low_pass);
        let rch = 1.0 / (2.0 * PI as f32 * high_pass);
        let delay = (delay_ms * SAMPLE_RATE as u64) as f32 / 1000.0;
        DelayLine {
            mem: vec![0.0; DELAY_SIZE as usize],
            current: 0,
            delay,
            alpha: dt / (rcl + dt),
            beta: rch / (rch + dt),
            lp: 0.0,
            hp: 0.0,
            osc: Osc::new(mod_freq),
            heads,
        }
    }

    fn add_delay(&mut self, sample: f32, delay: f32) {
        let d = delay * GLOBAL_DELAY_TIME + MOD_DEPTH * (2.0 + self.osc.sin);
        let f = d.floor() as usize;
        let r = d - d.floor();
        let i = (self.current + f) & (DELAY_SIZE - 1);
        let j = (self.current + 1 + f) & (DELAY_SIZE - 1);
        self.mem[i] += sample * (1.0 - r);
        self.mem[j] += sample * (r);
    }

    pub fn add_sample(&mut self, mut sample: f32) {
        let mut delay = self.delay;
        for i in 0..self.heads {
            self.add_delay(
                ((2 * (self.heads - i - 1) + self.heads) as f32 * sample)
                    / (2 * self.heads * (self.heads + 1)) as f32,
                delay,
            );
            delay = (delay * PSI);
            sample = -sample;
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        self.osc.inc();

        let sample = self.mem[self.current as usize];
        self.mem[self.current as usize] = 0.0;
        self.current = (self.current + 1) % (DELAY_SIZE);
        let l = self.lp;
        self.lp += (sample - self.lp) * self.alpha;
        self.hp = self.beta * (self.hp + self.lp - l);
        self.hp
    }
}

#[derive(Default)]
struct Svf {
    sr: f32,
    fc: f32,
    res: f32,
    drive: f32,
    freq: f32,
    damp: f32,
    notch: f32,
    low: f32,
    high: f32,
    band: f32,
    peak: f32,
    input: f32,
    outnotch: f32,
    outlow: f32,
    outhigh: f32,
    outpeak: f32,
    outband: f32,
}

impl Svf {
    fn new(samplerate: f32, freq: f32, res: f32) -> Self {
        let mut s = Svf::default();
        s.sr = samplerate;
        s.set_freq(freq);
        s.set_res(res);
        s
    }

    pub fn process(&mut self, input: f32) {
        self.input = input;
        // first pass
        self.notch = self.input - self.damp * self.band;
        self.low = self.low + self.freq * self.band;
        self.high = self.notch - self.low;
        self.band =
            self.freq * self.high + self.band - self.drive * self.band * self.band * self.band;
        self.outlow = 0.5 * self.low;
        self.outhigh = 0.5 * self.high;
        self.outband = 0.5 * self.band;
        self.outpeak = 0.5 * (self.low - self.high);
        self.outnotch = 0.5 * self.notch;
        // second pass
        self.notch = self.input - self.damp * self.band;
        self.low = self.low + self.freq * self.band;
        self.high = self.notch - self.low;
        self.band =
            self.freq * self.high + self.band - self.drive * self.band * self.band * self.band;
        self.outlow += 0.5 * self.low;
        self.outhigh += 0.5 * self.high;
        self.outband += 0.5 * self.band;
        self.outpeak += 0.5 * (self.low - self.high);
        self.outnotch += 0.5 * self.notch;
    }

    fn set_freq(&mut self, freq: f32) {
        if freq < 0.000001 {
            self.fc = 0.000001;
        } else if freq > self.sr / 2.0 {
            self.fc = (self.sr / 2.0) - 1.0;
        } else {
            self.fc = freq;
        }
        // Set Internal Frequency for self.fc
        self.freq = 2.0 * (std::f32::consts::PI * (self.fc / (self.sr * 2.0)).min(0.25)).sin(); // fs*2 because double sampled
                                                                                                // recalculate damp
                                                                                                //damp = (MIN(2.0f * powf(self.res, 0.25f), MIN(2.0f, 2.0f / freq - freq * 0.5f)));
        self.set_damp();
    }

    fn set_res(&mut self, mut r: f32) {
        if r < 0.0 {
            r = 0.0;
        } else if r > 1.0 {
            r = 1.0;
        }
        self.res = r;
        // recalculate damp
        //damp = (MIN(2.0f * powf(self.res, 0.25f), MIN(2.0f, 2.0f / freq - freq * 0.5f)));
        self.set_damp();
    }

    fn set_damp(&mut self) {
        self.damp =
            (2.0 * (1.0 - self.res.powf(0.25))).min((2.0 / self.freq - self.freq * 0.5).min(2.0));
    }
}
