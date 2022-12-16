use bevy::{
    ecs::{schedule::StateData, system::Resource},
    prelude::*,
};
pub use global_state_macros::{GlobalState, TransientState};
use std::marker::PhantomData;

pub trait GlobalState {
    fn init_global_state(app: &mut App);
}

pub trait AddGlobalState {
    fn add_global_state<T: GlobalState + Default + StateData>(&mut self) -> &mut Self;
}

impl AddGlobalState for App {
    fn add_global_state<T: GlobalState + Default + StateData>(&mut self) -> &mut Self {
        T::init_global_state(self);
        self
    }
}

pub trait TransientState {
    fn init_transient_state(app: &mut App);
}

pub trait AddTransientState {
    fn add_transient_state<T: TransientState + Default + StateData>(&mut self) -> &mut Self;
}

impl AddTransientState for App {
    fn add_transient_state<T: TransientState + Default + StateData>(&mut self) -> &mut Self {
        T::init_transient_state(self);
        self
    }
}

#[derive(Component)]
pub struct Persistent;

#[derive(Component)]
pub struct Transient;

#[derive(Default, Resource)]
pub struct StateTime<T>
where
    T: Resource + ?Sized,
{
    pub time: f32,
    _phantom: PhantomData<T>,
}

impl<T> StateTime<T>
where
    T: Resource + ?Sized,
{
    pub fn just_entered(&self) -> bool {
        self.time < 0.1
    }
}

pub fn reset_state_time<T>(mut state_time: ResMut<StateTime<T>>)
where
    T: Resource + ?Sized,
{
    state_time.time = 0.;
}

pub fn update_state_time<T>(mut state_time: ResMut<StateTime<T>>, time: Res<Time>)
where
    T: Resource + ?Sized,
{
    state_time.time += time.delta_seconds();
}

pub fn global_cleanup(
    mut commands: Commands,
    query: Query<Entity, (Without<Persistent>, Without<Parent>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn cleanup_transient(mut commands: Commands, query: Query<Entity, (With<Transient>, Without<Parent>)>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
