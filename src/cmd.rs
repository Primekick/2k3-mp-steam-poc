use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Arg<T> {
    Value(T),
    Ptr(Box<Self>),
}

impl<T> Arg<T> {
    pub fn into_value(self) -> T {
        match self {
            Arg::Value(v) => v,
            Arg::Ptr(b) => b.into_value()
        }
    }
}

impl FromStr for Arg<String> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Value(String::from(s)))
    }
}

#[derive(Debug, Clone)]
pub enum Cmd {
    InitSteam,
    RunCallbacks,
    TriggerAchievement(Arg<String>),
}

impl FromStr for Cmd {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = s.split(' ').collect::<Vec<_>>();
        match tokens.as_slice() {
            [cmd] => match cmd {
                &"init_steam" => Ok(Self::InitSteam),
                &"run_callbacks" => Ok(Self::RunCallbacks),
                _ => Err(()),
            }
            [cmd, args @ ..] => match cmd {
                &"trigger_achievement" => Ok(Self::TriggerAchievement(args[0].parse::<Arg<String>>()?)),
                _ => Err(()),
            }
            _ => Err(())
        }
    }
}