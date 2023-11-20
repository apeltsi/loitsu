use uuid::Uuid;

pub fn uuid() -> Uuid {
    Uuid::new_v4()
}
