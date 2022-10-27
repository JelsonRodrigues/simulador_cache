mod cache;
mod memoria;
mod politica_substituicao;
mod historico_acessos;
mod argumentos;

use std::{env::{args, Args}, borrow::Borrow, process::exit, fs::File, io::Read};

use memoria::Memoria;
use cache::Cache;
use politica_substituicao::random::Random;
use argumentos::Argumentos;

use crate::historico_acessos::HistoricoAcessos;

fn main() {
    let argumentos = parse_args(args());
    
    let nivel_inferior = Box::new(RAM);
    let cache_l1_conf = Cache::new(
        argumentos.nsets,
        argumentos.bsize,
        argumentos.assoc,
        32,
        true,
        1,
        nivel_inferior
    );

    let mut arquivo_enderecos = File::open(argumentos.arquivo_entrada)
                                        .expect("Não foi possível abrir o arquivo\n");
    
    let mut buffer:[u8; 4] = [0 as u8, 0 as u8, 0 as u8, 0 as u8];
    
    if argumentos.substituicao == "R" {
        let mut cache_l1 = Random::new(cache_l1_conf);
        
        loop {
            let bytes_lidos = arquivo_enderecos.read(&mut buffer).unwrap();
            if bytes_lidos < buffer.len() { break; }
            let endereco = u32::from_be_bytes(buffer);
            cache_l1.read(endereco);
        }

        mostrar_resultados(argumentos.flag_saida, cache_l1.historico_acessos());
    }
    else {
        println!("Ainda não implementado a opção {}", argumentos.substituicao);
    }
    
}

pub struct RAM;

impl Memoria for RAM {
    fn read(&mut self, endereco:u32) -> Vec<i8> {
        vec![4]
    }
    fn write(&mut self, valor: i8, endereco:u32) {}
}

fn usage() -> String {
    String::from(
        "cache_simulator nsets bsize assoc substituicao flag_saida arquivo_entrada
        nsets -> número de conjuntos total da cache
        bsize -> tamanho do bloco, em bytes
        assoc -> associatividade utilizada
        substituicao -> politica de substituição a utilizar, R: random, L: LRU, F: fifo
        flag_saida -> altera o formato da saida para o padrão 1, ou livre 0
        arquivo_entrada -> arquivo com os endereços a serem testados, no formato binário, onde cada endereço é um inteiro de 32 bits
        
        Os valores de nsets, bsize e assoc devem ser potências de 2!!!
        "
    )
}

fn parse_args(args:Args) -> Argumentos {
    let argumentos:Vec<String> = args.collect();

    if argumentos.len() < 7 {
        println!("Não foi possível identificar todos os argumentos, certifique-se que está no formato");
        println!("{}", usage());
        exit(-1);
    }

    Argumentos {
        nsets:u32::from_str_radix(argumentos[1].borrow(), 10).expect("NÃO FOI POSSÍVEL LER NSETS"),
        bsize:u32::from_str_radix(argumentos[2].borrow(), 10).expect("NÃO FOI POSSÍVEL LER BSIZE"),
        assoc:u32::from_str_radix(argumentos[3].borrow(), 10).expect("NÃO FOI POSSÍVEL LER ASSOC"),
        substituicao:argumentos[4].to_string(),
        flag_saida: if u8::from_str_radix(argumentos[5].borrow(), 10).expect("NÃO FOI POSSÍVEL LER FLAG") == 1 {true} else {false},
        arquivo_entrada: argumentos[6].to_string()
    }
}

fn mostrar_resultados(flag_saida:bool, historico:&HistoricoAcessos){
    if flag_saida {
        println!("{} {:.2} {:.2} {:.2} {:.2} {:.2}", 
            historico.total_acessos(),
            historico.calcular_taxa_de_hit(),
            historico.calcular_taxa_de_miss(),
            historico.calcular_taxa_de_miss_compulsorio(),
            historico.calcular_taxa_de_miss_capacidade(),
            historico.calcular_taxa_de_miss_conflito(),
        )
    }
    else {
        println!("Total de acessos: {}", historico.total_acessos());
        println!("Taxa de hit: {:.2}%", historico.calcular_taxa_de_hit() * 100.0);
        println!("Taxa de miss: {:.2}%", historico.calcular_taxa_de_miss() * 100.0);
        println!("Taxa de miss Compulsorio: {:.2}%", historico.calcular_taxa_de_miss_compulsorio() * 100.0);
        println!("Taxa de miss Capacidade: {:.2}%", historico.calcular_taxa_de_miss_capacidade() * 100.0);
        println!("Taxa de miss Conflito: {:.2}%", historico.calcular_taxa_de_miss_conflito() * 100.0);
    }
}