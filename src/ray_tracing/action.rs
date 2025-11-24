use std::collections::HashSet;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
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
    /// 请求进行渲染, 尽管画面没变化.
    RequestRender,
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

    pub(crate) fn withdraw(&mut self, action: Action) {
        self.actions.remove(&action);
    }

    pub(crate) fn is_triggerred(&self, action: Action) -> bool {
        self.actions.contains(&action)
    }

    pub(crate) fn clear(&mut self) {
        self.actions.clear();
    }

    pub(crate) fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }
}
