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


