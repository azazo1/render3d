use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Action {
    CameraMoveForward,
    CameraMoveBackward,
    CameraMoveLeft,
    CameraMoveRight,
    CameraMoveUp,
    CameraMoveDown,
    /// 摄像机水平顺时针转动.
    CameraRotationCW,
    /// 摄像机水平逆时针转动.
    CameraRotationCCW,
    CameraRotationUp,
    CameraRotationDown,
}

#[derive(Debug)]
pub(crate) struct ActionManager {
    actions: HashSet<Action>,
}

impl ActionManager {
    pub(crate) fn new() -> Self {
        Self {
            actions: HashSet::new(),
        }
    }

    pub(crate) fn trigger(&mut self, action: Action) {
        self.actions.insert(action);
    }

    #[allow(dead_code)]
    pub(crate) fn withdraw(&mut self, action: Action) {
        self.actions.remove(&action);
    }

    pub(crate) fn is_triggerred(&self, action: Action) -> bool {
        self.actions.contains(&action)
    }

    pub(crate) fn clear(&mut self) {
        self.actions.clear();
    }
}
