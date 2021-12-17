use aoclib::{geometry::Point, parse};
use std::path::Path;

/// The triangular numbers compute the x position of a probe after n steps.
///
/// See <https://oeis.org/A000217>.
///
/// We just copied this implementation from day 7; we could import it, but it's
/// not really worth doing so.
fn triangular_number(n: i32) -> i32 {
    n * (n + 1) / 2
}

/// Inverse triangular number: given a target number, return `n` such that
/// `triangular_number(n) >= target`.
///
/// This is an application of the quadratic formula.
fn inverse_triangular_number(target: i32) -> i32 {
    ((-1.0 + (1.0 + 8.0 * target as f64).sqrt()) / 2.0).ceil() as i32
}

#[derive(Debug, Default, Clone, Copy)]
struct Probe {
    position: Point,
    velocity: Point,
}

impl Probe {
    fn with_velocity(mut self, velocity: Point) -> Self {
        self.velocity = velocity;
        self
    }

    /// Adjust position by a single step.
    fn step(mut self) -> Self {
        self.position += self.velocity;
        match self.velocity.x.cmp(&0) {
            std::cmp::Ordering::Less => self.velocity.x += 1,
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => self.velocity.x -= 1,
        }
        self.velocity.y -= 1;
        self
    }

    /// Compute the y position of a probe after a certain number of steps,
    /// without simulating all the steps.
    #[cfg(debug_assertions)]
    fn y_position(self, steps: i32) -> i32 {
        triangular_number(self.velocity.y) - triangular_number(self.velocity.y - steps)
            + self.position.y
    }

    /// Find the min x velocity which lands the probe in the target area.
    ///
    /// The probe's x position after `n` steps is a triangular number:
    ///
    ///   n * (n + 1) / 2
    ///
    /// We therefore need the lowest n which causes n to be in range.
    ///
    /// We could math this, but this is pretty unlikely to be an efficiency problem.
    fn find_min_x(low_x: i32, high_x: i32) -> i32 {
        let x = inverse_triangular_number(low_x);
        debug_assert!(triangular_number(x) >= low_x);
        debug_assert!(triangular_number(x) <= high_x);
        x
    }

    fn set_min_x(&mut self, low_x: i32, high_x: i32) {
        self.velocity.x = Self::find_min_x(low_x, high_x)
    }

    /// Find the max y velocity which lands the probe in the target area.
    ///
    /// The probe _always_ returns to the initial position at step
    /// `(y_velocity * 2) + 1` for positive `y_velocity`. The maximum velocity
    /// which falls within the target area is therefore the last for which
    /// `y_position((y_velocity * 2) + 2)` falls within the target area.
    ///
    /// We can observe from this that for any given positive velocity,
    /// the position at `y_position(velocity * 2 + 2) == -velocity - 1`.
    /// It's therefore straightforward that `v_max_y = -(low_y + 1)`.
    fn find_max_y(low_y: i32, _high_y: i32) -> i32 {
        assert!(
            low_y < 0,
            "this formula is not known to work for positive target areas"
        );
        let v_max_y = -(low_y + 1);

        #[cfg(debug_assertions)]
        {
            fn position(velocity: i32) -> i32 {
                Probe::default()
                    .with_velocity(Point::new(0, velocity))
                    .y_position((velocity * 2) + 2)
            }

            assert!(position(v_max_y) >= low_y);
            assert!(position(v_max_y) <= _high_y);
            assert!(position(v_max_y + 1) < low_y);
        }

        v_max_y
    }

    fn set_max_y(&mut self, low_y: i32, high_y: i32) {
        self.velocity.y = Self::find_max_y(low_y, high_y)
    }

    fn max_height(self) -> i32 {
        self.position.y
            + if self.velocity.y > 0 {
                triangular_number(self.velocity.y)
            } else {
                0
            }
    }
}

#[derive(Debug, Clone, Copy, parse_display::FromStr, parse_display::Display)]
#[display("target area: x={low_x}..{high_x}, y={low_y}..{high_y}")]
struct TargetArea {
    low_x: i32,
    high_x: i32,
    low_y: i32,
    high_y: i32,
}

impl TargetArea {
    fn contains(&self, point: Point) -> bool {
        (self.low_x..=self.high_x).contains(&point.x)
            && (self.low_y..=self.high_y).contains(&point.y)
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    for (idx, target_area) in parse::<TargetArea>(input)?.enumerate() {
        let mut probe = Probe::default();
        probe.set_min_x(target_area.low_x, target_area.high_x);
        probe.set_max_y(target_area.low_y, target_area.high_y);

        println!(
            "target area {}: max y position {} (initial velocity: {},{})",
            idx,
            probe.max_height(),
            probe.velocity.x,
            probe.velocity.y
        );
    }
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    for (idx, target_area) in parse::<TargetArea>(input)?.enumerate() {
        let low_x = Probe::find_min_x(target_area.low_x, target_area.high_x);
        let high_y = Probe::find_max_y(target_area.low_y, target_area.high_y);

        let mut count_workable_velocities = 0;
        for vx in low_x..=target_area.high_x {
            for vy in target_area.low_y..=high_y {
                let mut probe = Probe::default().with_velocity(Point::new(vx, vy));

                for _ in 0.. {
                    probe = probe.step();
                    if target_area.contains(probe.position) {
                        count_workable_velocities += 1;
                        break;
                    }
                    if probe.position.y < target_area.low_y {
                        break;
                    }
                }
            }
        }

        println!(
            "target area {}: workable velocities: {}",
            idx, count_workable_velocities,
        );
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
