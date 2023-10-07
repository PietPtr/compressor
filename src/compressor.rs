use crate::csv_debugging::SampleLogger;

pub struct Algo {
    envelope: f32,
    sample_rate: f32,
    pub logger: SampleLogger, // TODO: Kinda ugly making this public
}

#[derive(Debug)]
pub struct RawParameters {
    pub threshold: f32,
    pub ratio: f32,
    pub steepness: f32,
    pub attack: f32,
    pub release: f32,
}

impl Algo {
    pub fn new() -> Self {
        Self { 
            envelope: 0.0, 
            sample_rate: 48000.0, 
            logger: SampleLogger::new() 
        }
    }

    pub fn _reset(&mut self) {
        self.envelope = 0.0;
        self.sample_rate = 48000.0;
    }

    pub fn process_samples(
        &mut self, 
        sample: &mut f32, // TODO: make work for several channels
        p: RawParameters
    ) -> Result<(), &'static str> {
        let attack_slope = 1.0 / (self.sample_rate * p.attack);
        let release_slope = 1.0 / (self.sample_rate * p.release);

        let envelope_scaler = 1.0 / (1.0 - p.threshold);

        self.logger.write("sample", *sample)?;
        self.logger.write("sample.abs()", (*sample).abs())?;
        self.logger.write("envelope", self.envelope)?;
        self.logger.write("threshold", p.threshold)?;
        self.logger.write("-threshold", -p.threshold)?;

        let abs_sample = (*sample).abs();

        self.envelope = if abs_sample > self.envelope {
            (self.envelope + attack_slope).min(abs_sample)
        } else if abs_sample < self.envelope {
            (self.envelope - release_slope).max(abs_sample)
        } else {
            self.envelope
        };
        
        let ratio = 1.0 / (((self.envelope - p.threshold) * envelope_scaler) * (p.ratio - 1.0) + 1.0);

        let wet = if self.envelope > p.threshold && *sample > p.threshold {
            p.threshold + (*sample - p.threshold) * ratio
        } else if -self.envelope < -p.threshold && *sample < -p.threshold {
            -(p.threshold + (abs_sample - p.threshold) * ratio)
        } else {
            *sample
        };

        let sigmoid = |x: f32| 1.0 / (1.0 + (p.steepness * x).exp());

        let distance_from_threshold = p.threshold - abs_sample;

        let mix = sigmoid(distance_from_threshold);
        *sample = *sample * (1.0 - mix) + wet * mix;

        self.logger.write("mix", mix)?;
            self.logger.write("after", *sample)?;

        Ok(())
    }
}