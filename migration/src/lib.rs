pub use sea_orm_migration::prelude::*;

mod m20250316_204923_user;
mod m20250323_195423_ride;
mod m20250323_220823_tag_descriptor;
mod m20250323_224215_ride_tag;
mod m20250323_230053_tag_enum_option;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250316_204923_user::Migration),
            Box::new(m20250323_195423_ride::Migration),
            Box::new(m20250323_220823_tag_descriptor::Migration),
            Box::new(m20250323_224215_ride_tag::Migration),
            Box::new(m20250323_230053_tag_enum_option::Migration),
        ]
    }
}
