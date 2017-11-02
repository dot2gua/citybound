use kay::{ActorSystem, World};
use compact::CVec;
use stagemaster::UserInterfaceID;

mod time;

pub use self::time::{Instant, Ticks, Duration, TICKS_PER_SIM_MINUTE, TICKS_PER_SIM_SECOND,
                     TimeOfDay, TimeOfDayRange};

pub trait Simulatable {
    fn tick(&mut self, dt: f32, current_instant: Instant, world: &mut World);
}

pub trait Sleeper {
    fn wake(&mut self, current_instant: Instant, world: &mut World);
}

#[derive(Compact, Clone)]
pub struct Simulation {
    id: SimulationID,
    simulatables: CVec<SimulatableID>,
    current_instant: Instant,
    sleepers: CVec<(Instant, SleeperID)>,
}

impl Simulation {
    pub fn spawn(
        id: SimulationID,
        simulatables: &CVec<SimulatableID>,
        _: &mut World,
    ) -> Simulation {
        Simulation {
            id,
            simulatables: simulatables.clone(),
            current_instant: Instant::new(0),
            sleepers: CVec::new(),
        }
    }

    pub fn do_tick(&mut self, world: &mut World) {
        for simulatable in &self.simulatables {
            simulatable.tick(
                1.0 / (TICKS_PER_SIM_SECOND as f32),
                self.current_instant,
                world,
            );
        }
        while self.sleepers
            .last()
            .map(|&(end, _)| end < self.current_instant)
            .unwrap_or(false)
        {
            let (_, sleeper) = self.sleepers.pop().expect(
                "just checked that there are sleepers",
            );
            sleeper.wake(self.current_instant, world);
        }
        self.current_instant += Ticks(1);

        let time = TimeOfDay::from(self.current_instant).hours_minutes();

        UserInterfaceID::local_first(world).add_debug_text(
            "Time".to_owned().into(),
            format!("{:02}:{:02}", time.0, time.1).into(),
            [0.0, 0.0, 0.0, 1.0],
            false,
            world,
        );
    }

    pub fn wake_up_in(&mut self, remaining_ticks: Ticks, sleeper_id: SleeperID, _: &mut World) {
        let wake_up_at = self.current_instant + remaining_ticks;
        let maybe_idx = self.sleepers.binary_search_by_key(
            &wake_up_at.iticks(),
            |&(t, _)| -(t.iticks()),
        );
        let insert_idx = match maybe_idx {
            Ok(idx) | Err(idx) => idx,
        };
        self.sleepers.insert(insert_idx, (wake_up_at, sleeper_id));
    }
}

pub fn setup(system: &mut ActorSystem, simulatables: Vec<SimulatableID>) -> SimulationID {
    system.register::<Simulation>();

    auto_setup(system);

    SimulationID::spawn(simulatables.into(), &mut system.world())
}

mod kay_auto;
pub use self::kay_auto::*;
