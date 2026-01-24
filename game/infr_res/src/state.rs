use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfrState {
    Menu,
    Game,
}

#[macro_export]
macro_rules! when_state {
    ($type: ty, $name: ident) => {
        |state: bevy::prelude::Res<bevy::prelude::State<$type>>| *state.get() == <$type>::$name
    };
    ($name: ident) => {
        |state: bevy::prelude::Res<bevy::prelude::State<$crate::InfrState>>| {
            *state.get() == $crate::InfrState::$name
        }
    };
}
