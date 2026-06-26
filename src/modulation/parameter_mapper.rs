//! Parameter Mapper
//!
//! Maps deterministic state generator output to LLM parameter ranges.
//! Replaces "mood engine" with explicit mathematical mappings.

use crate::modulation::state_generator::StateGenerator;

/// Maps generator state to LLM API parameters
#[derive(Debug, Clone)]
pub struct ParameterMapper {
    temp_min: f32,
    temp_max: f32,
    max_tokens_min: u32,
    max_tokens_max: u32,
    top_p_min: f32,
    top_p_max: f32,
}

impl Default for ParameterMapper {
    fn default() -> Self {
        Self::with_ranges(0.3, 1.2, 256, 2048, 0.7, 0.95)
    }
}

impl ParameterMapper {
    /// Create with custom parameter ranges
    pub fn with_ranges(
        temp_min: f32,
        temp_max: f32,
        max_tokens_min: u32,
        max_tokens_max: u32,
        top_p_min: f32,
        top_p_max: f32,
    ) -> Self {
        Self {
            temp_min,
            temp_max,
            max_tokens_min,
            max_tokens_max,
            top_p_min,
            top_p_max,
        }
    }

    /// Map generator state to complete LLM parameters
    ///
    /// Uses x for temperature, y for max_tokens, z for top_p
    /// This creates coupled modulation where parameters move together
    /// based on the underlying state trajectory.
    pub fn map_state(&self, gen: &StateGenerator) -> LLMParameters {
        let x = gen.normalized_x();
        let y = gen.normalized_y();
        let z = gen.normalized_z();

        LLMParameters {
            temperature: self.temp_min + (x as f32) * (self.temp_max - self.temp_min),
            max_tokens: (self.max_tokens_min as f64 + y * (self.max_tokens_max - self.max_tokens_min) as f64)
                as u32,
            top_p: self.top_p_min + (z as f32) * (self.top_p_max - self.top_p_min),
        }
    }

    /// Map with exploration boost (adds to temperature only)
    ///
    /// Used when repetition is detected to increase diversity.
    pub fn map_with_exploration(&self, gen: &StateGenerator, boost: f64) -> LLMParameters {
        let mut params = self.map_state(gen);
        let current_temp = params.temperature;
        params.temperature = (current_temp + boost as f32).clamp(self.temp_min, self.temp_max);
        params
    }
}

/// LLM API parameters
#[derive(Debug, Clone, Copy)]
pub struct LLMParameters {
    /// Temperature: 0.0-2.0, controls randomness
    pub temperature: f32,
    /// Max tokens: 1-8192, controls response length
    pub max_tokens: u32,
    /// Top-p: 0.0-1.0, nucleus sampling threshold
    pub top_p: f32,
}

impl LLMParameters {
    /// Apply exploration boost by increasing temperature
    pub fn boost_temperature(&mut self, amount: f32) {
        self.temperature = (self.temperature + amount).min(2.0);
    }

    /// Apply conservation by reducing max_tokens
    pub fn conserve_tokens(&mut self, factor: f32) {
        self.max_tokens = (self.max_tokens as f32 * factor).max(256.0) as u32;
    }

    /// Check if parameters are within valid ranges
    pub fn validate(&self) -> Result<(), ParameterError> {
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(ParameterError::InvalidTemperature(self.temperature));
        }
        if self.max_tokens == 0 {
            return Err(ParameterError::ZeroMaxTokens);
        }
        if self.top_p < 0.0 || self.top_p > 1.0 {
            return Err(ParameterError::InvalidTopP(self.top_p));
        }
        Ok(())
    }
}

/// Parameter validation error
#[derive(Debug)]
pub enum ParameterError {
    InvalidTemperature(f32),
    ZeroMaxTokens,
    InvalidTopP(f32),
}

impl std::fmt::Display for ParameterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterError::InvalidTemperature(t) => write!(f, "temperature {} out of range [0,2]", t),
            ParameterError::ZeroMaxTokens => write!(f, "max_tokens must be > 0"),
            ParameterError::InvalidTopP(p) => write!(f, "top_p {} out of range [0,1]", p),
        }
    }
}

impl std::error::Error for ParameterError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper_produces_valid_parameters() {
        let gen = StateGenerator::new(0.5);
        let mapper = ParameterMapper::default();
        let params = mapper.map_state(&gen);
        params.validate().unwrap();
    }

    #[test]
    fn exploration_boost_increases_temperature() {
        let gen = StateGenerator::new(0.5);
        let mapper = ParameterMapper::default();
        let base = mapper.map_state(&gen);
        let boosted = mapper.map_with_exploration(&gen, 0.3);
        assert!(boosted.temperature > base.temperature);
    }

    #[test]
    fn invalid_temperature_detected() {
        let params = LLMParameters {
            temperature: -0.5,
            max_tokens: 100,
            top_p: 0.9,
        };
        assert!(params.validate().is_err());
    }
}
