//! Deterministic State Generator using Lorenz ODEs
//!
//! Generates smooth, non-repeating sequences for parameter modulation.
//! Not "chaos" — produces predictable, deterministic output from initial conditions.

/// 3D state sequence generator using Lorenz differential equations
///
/// The Lorenz system is a set of three coupled nonlinear ODEs:
/// dx/dt = σ(y-x)
/// dy/dt = x(ρ-z)-y
/// dz/dt = xy-βz
///
/// This implementation uses 4th-order Runge-Kutta integration for numerical stability.
#[derive(Debug, Clone, Copy)]
pub struct StateGenerator {
    /// Current position (x, y, z)
    position: (f64, f64, f64),

    /// ODE parameters (sigma, rho, beta)
    parameters: (f64, f64, f64),

    /// Integration timestep
    dt: f64,

    /// Step counter
    step: u64,
}

impl StateGenerator {
    /// Create new generator with default Lorenz parameters
    ///
    /// Default parameters (σ=10, ρ=28, β=8/3) are the classic Lorenz attractor
    /// values that produce the characteristic butterfly pattern.
    pub fn new(seed: f64) -> Self {
        let seed = if seed.is_finite() { seed } else { 0.506 };
        Self {
            position: (seed, seed + 0.001, seed + 0.002),
            parameters: (10.0, 28.0, 8.0 / 3.0),
            dt: 0.005,
            step: 0,
        }
    }

    /// Create with custom parameters
    pub fn with_parameters(
        seed: f64,
        sigma: f64,
        rho: f64,
        beta: f64,
        dt: f64,
    ) -> Self {
        let seed = if seed.is_finite() { seed } else { 0.506 };
        Self {
            position: (seed, seed + 0.001, seed + 0.002),
            parameters: (sigma, rho, beta),
            dt,
            step: 0,
        }
    }

    /// Advance the generator one step via 4th-order Runge-Kutta integration
    ///
    /// Returns the new (x, y, z) position after integration.
    pub fn step(&mut self) -> (f64, f64, f64) {
        let (x, y, z) = self.position;
        let (sigma, rho, beta) = self.parameters;
        let dt = self.dt;

        // RK4 integration for numerical stability
        let k1x = sigma * (y - x);
        let k1y = x * (rho - z) - y;
        let k1z = x * y - beta * z;

        let x2 = x + 0.5 * dt * k1x;
        let y2 = y + 0.5 * dt * k1y;
        let z2 = z + 0.5 * dt * k1z;
        let k2x = sigma * (y2 - x2);
        let k2y = x2 * (rho - z2) - y2;
        let k2z = x2 * y2 - beta * z2;

        let x3 = x + 0.5 * dt * k2x;
        let y3 = y + 0.5 * dt * k2y;
        let z3 = z + 0.5 * dt * k2z;
        let k3x = sigma * (y3 - x3);
        let k3y = x3 * (rho - z3) - y3;
        let k3z = x3 * y3 - beta * z3;

        let x4 = x + dt * k3x;
        let y4 = y + dt * k3y;
        let z4 = z + dt * k3z;
        let k4x = sigma * (y4 - x4);
        let k4y = x4 * (rho - z4) - y4;
        let k4z = x4 * y4 - beta * z4;

        self.position = (
            x + (dt / 6.0) * (k1x + 2.0 * k2x + 2.0 * k3x + k4x),
            y + (dt / 6.0) * (k1y + 2.0 * k2y + 2.0 * k3y + k4y),
            z + (dt / 6.0) * (k1z + 2.0 * k2z + 2.0 * k3z + k4z),
        );

        self.step += 1;
        self.position
    }

    /// Get current position without advancing
    pub fn position(&self) -> (f64, f64, f64) {
        self.position
    }

    /// Map x coordinate to [0, 1] range
    ///
    /// The Lorenz attractor orbits roughly in x ∈ [-20, 20].
    pub fn normalized_x(&self) -> f64 {
        ((self.position.0 + 20.0) / 40.0).clamp(0.0, 1.0)
    }

    /// Map y coordinate to [0, 1] range
    pub fn normalized_y(&self) -> f64 {
        ((self.position.1 + 20.0) / 40.0).clamp(0.0, 1.0)
    }

    /// Map z coordinate to [0, 1] range
    pub fn normalized_z(&self) -> f64 {
        ((self.position.2 + 20.0) / 40.0).clamp(0.0, 1.0)
    }

    /// Map current state to a parameter range
    ///
    /// Uses x coordinate for primary mapping.
    pub fn map_to_range(&self, min: f64, max: f64) -> f64 {
        min + self.normalized_x() * (max - min)
    }

    /// Get current step count
    pub fn step_count(&self) -> u64 {
        self.step
    }

    /// Adjust rho parameter (affects orbital topology)
    pub fn adjust_rho(&mut self, delta: f64) {
        self.parameters.1 = (self.parameters.1 + delta).clamp(18.0, 38.0);
    }

    /// Get current rho value
    pub fn rho(&self) -> f64 {
        self.parameters.1
    }

    /// Get current sigma value
    pub fn sigma(&self) -> f64 {
        self.parameters.0
    }

    /// Apply temporary perturbation to sigma (decays naturally via ODE dynamics)
    pub fn perturb_sigma(&mut self, amount: f64) {
        self.parameters.0 = (self.parameters.0 + amount).clamp(1.0, 30.0);
    }
}

/// Secondary state generator using Logistic Map
///
/// Simpler 1D generator that can be reseeded from the primary generator
/// for coupled dynamics.
#[derive(Debug, Clone, Copy)]
pub struct SecondaryGenerator {
    r: f64,
    x: f64,
}

impl SecondaryGenerator {
    /// Create new secondary generator
    pub fn new(seed: f64) -> Self {
        let seed = if seed.is_finite() { seed } else { 0.506 };
        Self {
            r: 3.99, // Parameter for chaotic dynamics
            x: seed.clamp(0.0001, 0.9999),
        }
    }

    /// Advance one step
    pub fn step(&mut self) -> f64 {
        self.x = self.r * self.x * (1.0 - self.x);
        self.x
    }

    /// Get current value without advancing
    pub fn current(&self) -> f64 {
        self.x
    }

    /// Reseed from normalized primary generator output
    pub fn reseed(&mut self, normalized_value: f64) {
        if normalized_value.is_finite() {
            self.x = normalized_value.clamp(0.0001, 0.9999);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_stays_bounded() {
        let mut gen = StateGenerator::new(0.506);
        for _ in 0..10_000 {
            let (x, y, z) = gen.step();
            assert!(x.abs() < 100.0, "x diverged: {}", x);
            assert!(y.abs() < 100.0, "y diverged: {}", y);
            assert!(z.abs() < 100.0, "z diverged: {}", z);
        }
    }

    #[test]
    fn normalized_output_in_range() {
        let mut gen = StateGenerator::new(0.506);
        for _ in 0..1_000 {
            gen.step();
            let nx = gen.normalized_x();
            let ny = gen.normalized_y();
            let nz = gen.normalized_z();
            assert!((0.0..=1.0).contains(&nx), "normalized_x out of range: {}", nx);
            assert!((0.0..=1.0).contains(&ny), "normalized_y out of range: {}", ny);
            assert!((0.0..=1.0).contains(&nz), "normalized_z out of range: {}", nz);
        }
    }

    #[test]
    fn map_to_range_produces_expected_bounds() {
        let gen = StateGenerator::new(0.506);
        let mapped = gen.map_to_range(0.3, 1.2);
        assert!(mapped >= 0.3 && mapped <= 1.2, "mapped value {} out of range", mapped);
    }

    #[test]
    fn secondary_generator_stays_in_unit_interval() {
        let mut gen = SecondaryGenerator::new(0.506);
        for _ in 0..10_000 {
            let v = gen.step();
            assert!((0.0..=1.0).contains(&v), "secondary generator out of [0,1]: {}", v);
        }
    }

    #[test]
    fn secondary_reseed_works() {
        let mut gen = SecondaryGenerator::new(0.506);
        gen.step();
        gen.reseed(0.75);
        assert!((gen.current() - 0.75).abs() < 0.01, "reseed did not set value");
    }

    #[test]
    fn rho_adjustment_clamps() {
        let mut gen = StateGenerator::new(0.506);
        gen.adjust_rho(100.0);
        assert_eq!(gen.rho(), 38.0, "rho should clamp at max");
        gen.adjust_rho(-200.0);
        assert_eq!(gen.rho(), 18.0, "rho should clamp at min");
    }

    #[test]
    fn non_finite_seed_uses_default() {
        let gen = StateGenerator::new(f64::NAN);
        assert!(gen.position().0.is_finite(), "x should be finite");
        assert!(gen.position().1.is_finite(), "y should be finite");
        assert!(gen.position().2.is_finite(), "z should be finite");
    }
}
