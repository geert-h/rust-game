use crate::components::light::Light;
use crate::objects::game_object::GameObject;

pub enum SceneObject {
    Light(Light),
    GameObject(GameObject),
}