pub mod permission;
pub mod group;


pub enum Order {
    Ascending,
    Descending
}


impl ToString for Order {
    fn to_string(&self) -> String {
        return match self {
            Order::Ascending => "ASC".to_string(),
            Order::Descending => "DESC".to_string(),
        };
    }
}