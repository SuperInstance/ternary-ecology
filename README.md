# ternary-ecology

**Ecological dynamics where every population is {-1, 0, +1}. Lotka-Volterra predator-prey, food webs, competitive exclusion, and trophic cascades — discretized to ternary states.**

## Why This Exists

Classical ecology uses continuous differential equations: the Lotka-Volterra equations describe how fox and rabbit populations oscillate over time. But real populations aren't continuous — they're discrete, and when you're managing a large ecosystem simulation (or modeling an agent system inspired by ecology), you often don't care about exact population numbers. You care about trends: is this species growing, stable, or declining?

That's exactly three states. +1 for growing, 0 for stable, -1 for declining. The ternary encoding captures the essential dynamics — who eats whom, who's thriving, who's dying — without the overhead of continuous population tracking.

This crate implements ternary ecology: food webs, Lotka-Volterra dynamics, competitive exclusion, mutualism, and trophic cascades. Every population is a single ternary digit. The dynamics are qualitative, not quantitative — and that's the point.

## The Key Insight

In continuous Lotka-Volterra, the equations are:

```
dx/dt = αx - βxy    (prey grows, gets eaten)
dy/dt = δxy - γy    (predator grows from eating, dies naturally)
```

Ternary Lotka-Volterra replaces derivatives with discrete transitions:

```
prey_new  = clamp(prey + prey_growth - predation × predator, -1, +1)
pred_new  = clamp(pred + pred × prey - pred_death, -1, +1)
```

Same structure, but every variable and every result is {-1, 0, +1}. The clamp replaces the continuous dynamics. You lose the beautiful oscillation diagrams (no limit cycles in a three-valued system), but you gain:
- **Instant computation** — no numerical integration needed
- **Exact dynamics** — no rounding, no step-size issues
- **Qualitative correctness** — the trends (who eats whom, who benefits) are preserved
- **Composability** — ternary populations plug directly into other ternary systems (graphs, networks, topology)

## Quick Start

```rust
use ternary_ecology::*;

// Build a food web with 3 species
let mut web = FoodWeb::new(3);
web.species[0].population = 1;   // Plants: growing
web.species[1].population = 0;   // Herbivores: stable
web.species[2].population = -1;  // Predators: declining

// Define who eats whom
web.add_predation(1, 0);  // Herbivores eat plants
web.add_predation(2, 1);  // Predators eat herbivores

// Simulate 10 time steps
let history = web.run(10);
for (step, pops) in history.iter().enumerate() {
    println!("Step {}: {:?}", step, pops);
}

// Check stability: did populations converge?
let stable = web.is_stable(&history);
println!("Ecosystem stable: {}", stable);

// What happens if we remove the predator? (trophic cascade)
let cascade = web.trophic_cascade(2, 5);
// Without predation pressure, herbivores should grow unchecked
```

### Two-Species Models

```rust
use ternary_ecology::*;

// Classic predator-prey
let (prey, pred) = lotka_volterra_2species(1, 0, 1, 1, 0);
// prey=1 (growing), pred=0 (stable)
// After one step: prey grows, predator gets nothing from empty prey

// Competitive exclusion: two species fight for same resource
let (sp1, sp2) = competitive_exclusion(1, 1, 1);
// Both growing, high competition → both decline
assert!(sp1 <= 0);
assert!(sp2 <= 0);

// Mutualism: two species help each other
let (sp1, sp2) = mutualism(1, 1, 1);
// Both benefit → both stay positive
assert_eq!(sp1, 1);
assert_eq!(sp2, 1);
```

## Architecture

### Core Types

| Type | Description |
|------|-------------|
| `Species` | An organism with ID, ternary population, and trophic level |
| `FoodWeb` | A directed graph of predation relationships with population dynamics |

### Species

```rust
pub struct Species {
    pub id: usize,
    pub population: i8,       // -1=declining, 0=stable, +1=growing
    pub trophic_level: i8,    // -1=decomposer, 0=primary, +1=predator
}
```

The trophic level is also ternary. -1 for decomposers (bottom of the food chain), 0 for primary producers (plants), +1 for predators. This gives a simple three-level food chain.

### FoodWeb Methods

| Method | Description |
|--------|-------------|
| `new(n_species)` | Create ecosystem with N species |
| `add_predation(predator, prey)` | Define predation link |
| `step()` | One time step of Lotka-Volterra dynamics |
| `run(steps)` | Run N steps, return population history |
| `is_stable(history)` | True if last 3 steps are identical |
| `diversity()` | Count of non-zero populations |
| `biomass()` | Sum of populations (clamped to {-1, 0, +1}) |
| `trophic_cascade(removed, steps)` | Remove a species, simulate cascade |

### Standalone Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `lotka_volterra_2species` | `(prey, pred, growth, predation, death) → (prey', pred')` | Two-species predator-prey |
| `competitive_exclusion` | `(sp1, sp2, competition) → (sp1', sp2')` | Two species, same niche |
| `mutualism` | `(sp1, sp2, benefit) → (sp1', sp2')` | Two species, mutual benefit |

### Predation Matrix

The predation matrix is the core data structure:

```
predation[i][j] = +1  →  species i eats species j
predation[i][j] = -1  →  species i is eaten by species j
predation[i][j] =  0  →  no interaction
```

Adding a predation link sets both directions: `add_predation(2, 0)` sets `predation[2][0] = 1` and `predation[0][2] = -1`.

### Population Dynamics

Each `step()`:

1. For each species, sum up interaction effects from all other species
2. If species i eats j: `growth += j.population` (more prey → better for predator)
3. If species i is eaten by j: `growth -= j.population` (more predators → worse for prey)
4. Add logistic term: growing species get -1 (carrying capacity), declining species get +1 (recovery)
5. Clamp: `new_population = (old + growth).clamp(-1, 1)`

The logistic term prevents any species from staying at +1 forever (carrying capacity) and provides a recovery mechanism for species at -1 (reproduction from remaining individuals).

## Real-World Example: Three-Level Food Chain

```rust
use ternary_ecology::*;

let mut web = FoodWeb::new(3);

// Trophic levels
web.species[0].trophic_level = 0;   // Plants (primary producer)
web.species[1].trophic_level = 1;   // Herbivores
web.species[2].trophic_level = 1;   // Predators

// Initial populations
web.species[0].population = 1;   // Plants abundant
web.species[1].population = 1;   // Herbivores thriving
web.species[2].population = 0;   // Predators holding steady

// Food chain: predators eat herbivores, herbivores eat plants
web.add_predation(1, 0);  // Herbivores → Plants
web.add_predation(2, 1);  // Predators → Herbivores

// Run the ecosystem
let history = web.run(20);

// What happens when we remove plants? (bottom-up cascade)
let mut web2 = FoodWeb::new(3);
// ... same setup ...
let cascade = web2.trophic_cascade(0, 10);
// Without plants: herbivores lose food source → decline
// Without herbivores: predators lose food source → decline
// Total ecosystem collapse propagates up the food chain
```

## Design Decisions

**`#![no_std]`** — Ecological simulation shouldn't require an OS. Runs in embedded, WASM, and kernel contexts.

**Ternary populations** — The {-1, 0, +1} encoding captures trends, not numbers. You can't tell *how many* rabbits there are, only whether the population is rising, stable, or falling. For many applications (monitoring, early warning, qualitative modeling), this is exactly what you need.

**Clamping instead of mod-3** — Population updates are clamped to [-1, +1], not wrapped mod 3. A population of +1 (growing) that should grow further stays at +1 (carrying capacity), it doesn't wrap to -1. This is ecological, not algebraic.

**Symmetric predation** — `add_predation(predator, prey)` automatically sets the reverse link. The predation matrix is always antisymmetric: if i eats j, j is eaten by i. No manual bookkeeping.

## Ecosystem Connections

- **`ternary-graph`** — Food webs are ternary-weighted directed graphs
- **`ternary-pagerank`** — Centrality on food webs (who's the keystone species?)
- **`ternary-topology`** — Self-organizing topology (species as nodes, predation as edges)
- **`ternary-automata`** — Cellular automata (spatial ecology on grids)
- **`ternary-grid`** — Spatial grids for ecosystem simulation
- **`ternary-network`** — Network science measures for ecological networks

## Open Questions

- **Spatial ecology**: The current model is well-mixed (every species interacts with every other). Real ecosystems have spatial structure — plants grow in patches, predators have territories. A grid-based extension using `ternary-grid` would model this.
- **Oscillation**: Continuous Lotka-Volterra produces beautiful predator-prey oscillations. Ternary dynamics can't oscillate in the same way (finite state). Can the ternary model capture the *qualitative* oscillation (alternating growth/decline)?
- **Higher trophic levels**: Three trophic levels is standard. Can the model handle more complex food webs (omnivory, cannibalism, detritivory)?
- **Stochastic dynamics**: The current model is deterministic. Adding noise (via a PRNG seed) would model environmental stochasticity.

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | ~274 |
| Tests | 13 |
| Public API | 14 items |
| Dependencies | 0 (no_std) |

## License

Apache-2.0
