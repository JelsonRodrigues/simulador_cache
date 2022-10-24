use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use crate::historico_acessos::{FonteMiss, HistoricoAcessos};
use crate::cache::Cache;
use crate::memoria::Memoria;

struct Linha {
    validade:bool,
    tag:u32,
    bloco:Vec<i8>,
}
impl Linha {
    fn new() -> Self {
        Linha { validade: false, tag: 0, bloco: Vec::new() }
    }
}

pub struct Random {
    cache_conf:Cache,
    memoria:Vec<Vec<Linha>>,
    random_source:ThreadRng,
}

impl Random {
    pub fn new(cache_conf: Cache) -> Self { 
        let mut memoria = Vec::with_capacity(cache_conf.nsets() as usize);
        
        for _ in 0..cache_conf.nsets() {
            let mut item:Vec<Linha> = Vec::with_capacity(cache_conf.assoc() as usize);
            for _ in 0..cache_conf.assoc() {
                let mut bloco = Linha::new();
                for _ in 0..cache_conf.bsize() {
                    bloco.bloco.push(0);
                }
                item.push(bloco);
            }
            memoria.push(item);  
        }

        let random_source = thread_rng();

        Self { 
            cache_conf, 
            memoria, 
            random_source 
        } 
    }
    
    pub fn historico_acessos(&self) -> &HistoricoAcessos {
        self.cache_conf.historico()
    }
    
    fn trata_falta_leitura(&mut self, endereco:u32) {
        // Lê do nível inferior a nova linha
        let mut nova_linha = Linha::new();
        nova_linha.tag = self.cache_conf.tag_do_endereco(endereco);
        nova_linha.validade = true;

        let tamanho_bloco = self.cache_conf.bsize();
        let endereco_sem_offset = self.cache_conf.offset_endereco(endereco) ^ endereco;
        for i in 0..tamanho_bloco {
            let mut byte = self.cache_conf.nivel_inferior_mut().read(endereco_sem_offset + i);
            
            nova_linha.bloco.append(&mut byte);
        }

        // Verifica se tem algum espaço vazio no conjunto para inserir
        let linha_conjunto = self.cache_conf.indice_do_endereco(endereco);
        let possui_espaco_nao_ocupado = self.possui_espaco_vazio_no_conjunto(linha_conjunto);

        // Se sim, insere no local
        if possui_espaco_nao_ocupado.0 {
            self.memoria[linha_conjunto as usize][possui_espaco_nao_ocupado.1] = nova_linha;
        }
        // Se não aleatoriamente escolhe o valor a ser substituido
        else {
            let posicao:u32 = self.random_source.gen();
            self.memoria[linha_conjunto as usize][(posicao % self.cache_conf.assoc()) as usize] = nova_linha;
        }
    }
    fn possui_espaco_vazio_no_conjunto(&self, indice:u32) -> (bool, usize) {
        let mut posicao = 0;
        for c in &self.memoria[indice as usize] {
            if c.validade {
                return (true, posicao);
            }
            posicao += 1;
        }
        return (false, posicao);
    }
}

impl Memoria for Random {
    fn read(&mut self, endereco:u32) -> Vec<i8> {
        // Verifica se o item está na cache, se estiver atualiza o histórico de hits/miss
        let indice_procura = self.cache_conf.indice_do_endereco(endereco);
        let tag_procura = self.cache_conf.tag_do_endereco(endereco);
        let mut resultado:Vec<i8> = Vec::new();

        for item in &self.memoria[indice_procura as usize] {
            if item.tag == tag_procura && item.validade {
                self.cache_conf.historico_mut().adicionar_hit();
                // Se endereçado a byte, retorna um único byte
                // Senão retorna o número de bytes correspondente ao tamanho do endereço
                if self.cache_conf.enderecado_byte() {
                    resultado.push(item.bloco.get(self.cache_conf.offset_endereco(endereco) as usize).unwrap().clone());
                }
                else {
                    // Aqui eu vou ter que ver depois qual a parte do bloco eu tenho que pegar
                    // o retorno deve ser sempre: tamanho do endereço em bits / 8 bits
                    for valor in &item.bloco {
                        resultado.push(valor.clone());
                    }
                }
                break;
            }
        }

        if resultado.len() == 0 {
            // Occoreu um miss
            
            // Mapeamento direto
            if self.cache_conf.assoc() == 1 {
                if self.memoria[indice_procura as usize][0].validade == false {
                    self.cache_conf.historico_mut().adicionar_miss(FonteMiss::Compulsorio);
                }
                else {
                    self.cache_conf.historico_mut().adicionar_miss(FonteMiss::Conflito);
                }
            }
            else {
                let indice_vazio = self.possui_espaco_vazio_no_conjunto(indice_procura);
                if indice_vazio.0 {
                    self.cache_conf.historico_mut().adicionar_miss(FonteMiss::Compulsorio);
                }
                else {
                    self.cache_conf.historico_mut().adicionar_miss(FonteMiss::Capacidade);
                }
            }

            self.trata_falta_leitura(endereco);

            for item in &self.memoria[indice_procura as usize] {
                if item.tag == tag_procura && item.validade {
                    // Se endereçado a byte, retorna um único byte
                    // Senão retorna o número de bytes correspondente ao tamanho do endereço
                    if self.cache_conf.enderecado_byte() {
                        resultado.push(item.bloco.get(self.cache_conf.offset_endereco(endereco) as usize).unwrap().clone());
                    }
                    else {
                        // Aqui eu vou ter que ver depois qual a parte do bloco eu tenho que pegar
                        // o retorno deve ser sempre: tamanho do endereço em bits / 8 bits
                        for valor in &item.bloco {
                            resultado.push(valor.clone());
                        }
                    }
                    break;
                }
            }
        }
        
        resultado
    }
    fn write(&mut self, valor: i8, endereco:u32) {
        
    }
}
