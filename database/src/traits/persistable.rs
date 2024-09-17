pub trait Persistable {
    fn save(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn load(id: uuid::Uuid) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}
