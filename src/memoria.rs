pub trait Memoria {
    fn read(&mut self, endereco:u32) -> Vec<i8>;
    fn write(&mut self, valor: i8, endereco:u32);
}