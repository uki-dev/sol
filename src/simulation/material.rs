#[derive(Debug, Clone, Copy)]
pub enum Material {
    Air,
    Water,
    Sand,
    Soil,
}
unsafe impl Zeroable for Material {}
unsafe impl Pod for Material {}
impl From<u32> for Material {
    fn from(value: u32) -> Self {
        match value {
            0 => Material::Air,
            1 => Material::Water,
            2 => Material::Sand,
            3 => Material::Soil,
            _ => panic!("Invalid u8 value for Material"),
        }
    }
}
