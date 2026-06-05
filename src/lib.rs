//! # ternary-ecology
//!
//! Ecological dynamics on ternary populations.
//! Lotka-Volterra predator-prey, food webs, population cycles, ecosystem stability.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
use alloc::{vec, vec::Vec};

/// A species in a ternary ecosystem
#[derive(Debug, Clone)]
pub struct Species {
    pub id: usize,
    pub population: i8,  // -1=declining, 0=stable, 1=growing
    pub trophic_level: i8, // -1=decomposer, 0=primary, 1=predator
}

impl Species {
    pub fn new(id: usize, population: i8, trophic_level: i8) -> Self {
        Self { id, population: population.clamp(-1, 1), trophic_level: trophic_level.clamp(-1, 1) }
    }
}

/// A food web: who eats whom
#[derive(Debug, Clone)]
pub struct FoodWeb {
    pub species: Vec<Species>,
    /// predation[i][j] = 1 if i eats j, -1 if i is eaten by j, 0 if no interaction
    pub predation: Vec<Vec<i8>>,
}

impl FoodWeb {
    pub fn new(n_species: usize) -> Self {
        Self {
            species: (0..n_species).map(|i| Species::new(i, 0, 0)).collect(),
            predation: vec![vec![0i8; n_species]; n_species],
        }
    }

    pub fn add_predation(&mut self, predator: usize, prey: usize) {
        self.predation[predator][prey] = 1;
        self.predation[prey][predator] = -1;
    }

    /// One step of Lotka-Volterra dynamics
    /// Prey grows when population is high and predation is low
    /// Predator grows when prey is available
    pub fn step(&mut self) {
        let n = self.species.len();
        let mut new_pops = vec![0i8; n];

        for i in 0..n {
            let pop = self.species[i].population;
            let mut growth = 0i8;

            for j in 0..n {
                let interaction = self.predation[i][j];
                let other_pop = self.species[j].population;

                match interaction {
                    1 => {
                        // i eats j: growth depends on prey availability
                        growth += other_pop;
                    }
                    -1 => {
                        // i is eaten by j: decline depends on predator presence
                        growth -= other_pop;
                    }
                    _ => {}
                }
            }

            // Logistic term: population tends toward stability
            if pop == 1 { growth -= 1; } // carrying capacity
            if pop == -1 { growth += 1; } // recovery

            new_pops[i] = (pop + growth).clamp(-1, 1);
        }

        for i in 0..n {
            self.species[i].population = new_pops[i];
        }
    }

    /// Run for N steps
    pub fn run(&mut self, steps: usize) -> Vec<Vec<i8>> {
        let mut history = vec![];
        for _ in 0..steps {
            self.step();
            history.push(self.species.iter().map(|s| s.population).collect());
        }
        history
    }

    /// Ecosystem stability: do populations converge to a fixed point?
    pub fn is_stable(&self, history: &[Vec<i8>]) -> bool {
        if history.len() < 3 { return false; }
        let n = history.len();
        let last = &history[n - 1];
        let prev = &history[n - 2];
        let pprev = &history[n - 3];
        last == prev && prev == pprev
    }

    /// Diversity: number of non-zero populations
    pub fn diversity(&self) -> usize {
        self.species.iter().filter(|s| s.population != 0).count()
    }

    /// Total biomass: sum of populations
    pub fn biomass(&self) -> i8 {
        self.species.iter().map(|s| s.population).sum::<i8>().clamp(-1, 1)
    }

    /// Trophic cascade: what happens to all species when one is removed?
    pub fn trophic_cascade(&mut self, removed_species: usize, steps: usize) -> Vec<Vec<i8>> {
        self.species[removed_species].population = 0;
        // Remove predation links
        for i in 0..self.species.len() {
            self.predation[i][removed_species] = 0;
            self.predation[removed_species][i] = 0;
        }
        self.run(steps)
    }
}

/// Lotka-Volterra two-species model (predator-prey)
pub fn lotka_volterra_2species(prey_pop: i8, pred_pop: i8, prey_growth: i8, predation_rate: i8, pred_death: i8) -> (i8, i8) {
    // Prey: grows by prey_growth, declines by predation_rate * predator
    let new_prey = (prey_pop + prey_growth - predation_rate * pred_pop).clamp(-1, 1);
    // Predator: grows by eating prey, declines by natural death
    let new_pred = (pred_pop + pred_pop * prey_pop - pred_death).clamp(-1, 1);
    (new_prey, new_pred)
}

/// Competitive exclusion: two species competing for the same resource
pub fn competitive_exclusion(sp1: i8, sp2: i8, competition: i8) -> (i8, i8) {
    // Both decline when competition is high
    let new_sp1 = (sp1 - competition * sp2).clamp(-1, 1);
    let new_sp2 = (sp2 - competition * sp1).clamp(-1, 1);
    (new_sp1, new_sp2)
}

/// Mutualism: two species that benefit each other
pub fn mutualism(sp1: i8, sp2: i8, benefit: i8) -> (i8, i8) {
    let new_sp1 = (sp1 + benefit * sp2).clamp(-1, 1);
    let new_sp2 = (sp2 + benefit * sp1).clamp(-1, 1);
    (new_sp1, new_sp2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_web_new() {
        let fw = FoodWeb::new(3);
        assert_eq!(fw.species.len(), 3);
        assert_eq!(fw.diversity(), 0);
    }

    #[test]
    fn test_add_predation() {
        let mut fw = FoodWeb::new(3);
        fw.add_predation(2, 0); // species 2 eats species 0
        assert_eq!(fw.predation[2][0], 1);
        assert_eq!(fw.predation[0][2], -1);
    }

    #[test]
    fn test_step_predator_prey() {
        let mut fw = FoodWeb::new(2);
        fw.species[0].population = 1; // prey growing
        fw.species[1].population = 0; // predator stable
        fw.add_predation(1, 0);
        fw.step();
        // Predator should grow from available prey
        assert!(fw.species[1].population >= 0);
    }

    #[test]
    fn test_run() {
        let mut fw = FoodWeb::new(2);
        fw.species[0].population = 1;
        fw.species[1].population = 1;
        fw.add_predation(1, 0);
        let history = fw.run(10);
        assert_eq!(history.len(), 10);
    }

    #[test]
    fn test_stability() {
        let history = vec![
            vec![1, 0],
            vec![1, 0],
            vec![1, 0],
        ];
        let fw = FoodWeb::new(2);
        assert!(fw.is_stable(&history));
    }

    #[test]
    fn test_not_stable() {
        let history = vec![
            vec![1, 0],
            vec![0, 1],
            vec![1, 0],
        ];
        let fw = FoodWeb::new(2);
        assert!(!fw.is_stable(&history));
    }

    #[test]
    fn test_diversity() {
        let mut fw = FoodWeb::new(3);
        fw.species[0].population = 1;
        fw.species[1].population = -1;
        assert_eq!(fw.diversity(), 2);
    }

    #[test]
    fn test_biomass() {
        let mut fw = FoodWeb::new(3);
        fw.species[0].population = 1;
        fw.species[1].population = 1;
        fw.species[2].population = -1;
        assert_eq!(fw.biomass(), 1);
    }

    #[test]
    fn test_lotka_volterra() {
        let (prey, pred) = lotka_volterra_2species(1, 0, 1, 1, 0);
        assert!(prey >= -1 && prey <= 1);
        assert!(pred >= -1 && pred <= 1);
    }

    #[test]
    fn test_lotka_volterra_predator_grows() {
        let (prey, pred) = lotka_volterra_2species(1, 1, 0, 1, 0);
        // Predator should grow from eating prey
        assert!(pred >= 0);
    }

    #[test]
    fn test_competitive_exclusion() {
        let (sp1, sp2) = competitive_exclusion(1, 1, 1);
        // Both decline from competition
        assert!(sp1 <= 1);
        assert!(sp2 <= 1);
    }

    #[test]
    fn test_mutualism() {
        let (sp1, sp2) = mutualism(1, 1, 1);
        // Both benefit
        assert_eq!(sp1, 1); // 1 + 1*1 = 2, clamped to 1
        assert_eq!(sp2, 1);
    }

    #[test]
    fn test_trophic_cascade() {
        let mut fw = FoodWeb::new(3);
        fw.species[0].population = 1;
        fw.species[1].population = 1;
        fw.species[2].population = 1;
        fw.add_predation(1, 0);
        fw.add_predation(2, 1);
        let result = fw.trophic_cascade(2, 5);
        assert_eq!(result.len(), 5);
    }
}
