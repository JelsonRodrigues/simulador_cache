mod cache;
mod memoria;
mod politica_substituicao;
mod historico_acessos;

use memoria::Memoria;
use cache::Cache;
use politica_substituicao::random::Random;

fn main() {
    let nivel_inferior = Box::new(RAM);
    let cache_l1_conf = Cache::new(32, 4, 1, 32, true, nivel_inferior);
    let mut cache_l1 = Random::new(cache_l1_conf);
    cache_l1.read(15);
    cache_l1.read(268);
    cache_l1.read(140);
    cache_l1.read(480);
    cache_l1.read(15);
    cache_l1.read(14);
    cache_l1.read(15);
    cache_l1.read(140);
    let v = cache_l1.historico_acessos();
    println!("Taxa de hit: {}", v.calcular_taxa_de_hit());
    println!("Taxa de miss: {}", v.calcular_taxa_de_miss());
    println!("Taxa de miss Compulsorio: {}", v.calcular_taxa_de_miss_compulsorio());
    println!("Taxa de miss Capacidade: {}", v.calcular_taxa_de_miss_capacidade());
    println!("Taxa de miss Conflito: {}", v.calcular_taxa_de_miss_conflito());
}

pub struct RAM;

impl Memoria for RAM {
    fn read(&mut self, endereco:u32) -> Vec<i8> {
        vec![1]
    }
    fn write(&mut self, valor: i8, endereco:u32) {}
}