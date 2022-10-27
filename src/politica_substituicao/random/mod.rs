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
    fn new(cache_conf:&Cache) -> Self {
        Linha { validade: false, tag: 0, bloco: Linha::criar_bloco(cache_conf.bsize() as usize) }
    }
    fn criar_bloco(bsize:usize) -> Vec<i8> {
        // Retorna um vetor com bsize posicoes, cada posicao tem tipo i8 e esta inicializada com 0
        vec![0 as i8;bsize]
    }
}

pub struct Random {
    /*
        Contem as informacoes da cache, tais como nsets, assoc, bsize ...
        Tambem contem um registro dos acessos a cache,  que guardam a informacao
        de numero de hits, misses e o tipo dos misses
    */
    cache_conf:Cache,
    
    /*
        E uma matriz, onde o numero de linhas da matriz
        sera o valor de nsets, o numero de colunas sera
        a associatividade, e cada item dentro da matriz sera
        do tipo Linha, que possui tag, validade e um vetor 
        de bsize posicoes.
        Deste modo e uma matriz tridimensional, A x B x C, 
        onde A = nsets, B = assoc e C = bsize
    */
    memoria:Vec<Vec<Linha>>,

    /*
        Fonte de números pseudo-aleatórios
    */
    random_source:ThreadRng,

    cache_cheia:bool,
}

impl Random {
    // Construtor
    pub fn new(cache_conf: Cache) -> Self { 
        // Numero de linhas na matriz
        let mut memoria = Vec::with_capacity(cache_conf.nsets() as usize);
        
        for _ in 0..cache_conf.nsets() {
            // Numero de colunas da matriz
            let mut item:Vec<Linha> = Vec::with_capacity(cache_conf.assoc() as usize);
            for _ in 0..cache_conf.assoc() {
                // Criacao do conteudo de cada bloco da cache
                let bloco = Linha::new(&cache_conf);
                item.push(bloco);
            }
            memoria.push(item);  
        }

        let random_source = thread_rng();

        Self { 
            cache_conf, 
            memoria, 
            random_source,
            cache_cheia:false
        } 
    }
    
    // Retorna uma referencia de read-only do historico de acessos da cache
    pub fn historico_acessos(&self) -> &HistoricoAcessos {
        self.cache_conf.historico()
    }
    
    fn trata_falta_leitura(&mut self, endereco:u32) -> Linha {
        // Cria a nova linha que entrara na cache
        let mut nova_linha = Linha::new(&self.cache_conf);
        nova_linha.tag = self.cache_conf.tag_do_endereco(endereco);
        nova_linha.validade = true;
        nova_linha.bloco.clear();

        let tamanho_bloco = self.cache_conf.bsize();
        let endereco_sem_offset = self.cache_conf.offset_endereco(endereco) ^ endereco;
        
        if self.cache_conf.enderecado_byte() {
            // Busca o número de bytes da memória inferior para encher o bloco
            for i in 0..tamanho_bloco {
                let mut byte = self.cache_conf.nivel_inferior_mut().read(endereco_sem_offset + i);
    
                nova_linha.bloco.append(&mut byte);
            }
        }
        else {
            // Busca o número de palavras da memória inferior para encher o bloco
            let palavras_por_bloco = tamanho_bloco / self.cache_conf.bytes_por_palavra() as u32;
            for i in 0..palavras_por_bloco {
                let mut byte = self.cache_conf.nivel_inferior_mut().read(endereco_sem_offset + i);
    
                nova_linha.bloco.append(&mut byte);
            }
        }

        nova_linha
    }
    /*
    Esta funcao procura por algum bloco vazio na linhas, e retorna uma tupla
    que contem uma flag true/false se encontrou ou nao e caso seja true, 
    o segundo elemento da tupla e o indice da coluna da matriz, onde esta vazio
    */
    fn possui_espaco_vazio_no_conjunto(&self, indice:u32) -> (bool, usize) {
        let mut posicao = 0;
        for c in &self.memoria[indice as usize] {
            if c.validade == false {
                return (true, posicao);
            }
            posicao += 1;
        }
        return (false, posicao);
    }
    /*
    Passa por cada linha e verifica se existe espaço vazio em algum conjunto da linha
    */
    fn verifica_cache_encheu(&self) -> bool {
        let mut cheia = true;
        for i in 0..self.memoria.len() as u32 {
            if self.possui_espaco_vazio_no_conjunto(i).0 {
                cheia = false;
                break;
            }
        }
        cheia
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
                if self.cache_conf.enderecado_byte() {
                    resultado.push(item.bloco.get(self.cache_conf.offset_endereco(endereco) as usize).unwrap().clone());
                }
                else {
                    let inicio_palavra = self.cache_conf.offset_endereco(endereco) as usize;
                    for i in 0..self.cache_conf.bytes_por_palavra() as usize {
                        resultado.push(item.bloco.get(inicio_palavra + i).unwrap().clone());
                    }
                }
                break;
            }
        }

        // Occoreu um miss ?
        if resultado.len() == 0 {
            
            let nova_linha = self.trata_falta_leitura(endereco);

            // Copia os valores para o resultado
            if self.cache_conf.enderecado_byte() {
                resultado.push(nova_linha.bloco.get(self.cache_conf.offset_endereco(endereco) as usize).unwrap().clone());
            }
            else {
                let inicio_palavra = self.cache_conf.offset_endereco(endereco) as usize;
                for i in 0..self.cache_conf.bytes_por_palavra() as usize {
                    resultado.push(nova_linha.bloco.get(inicio_palavra + i).unwrap().clone());
                }
            }


            // Verifica se tem algum conjunto com espaço vazio para inserir na linha da cache
            let possui_espaco_nao_ocupado = self.possui_espaco_vazio_no_conjunto(indice_procura);

            // Se sim, insere no local e  adiciona um miss compulsório
            if possui_espaco_nao_ocupado.0 {
                self.memoria[indice_procura as usize][possui_espaco_nao_ocupado.1] = nova_linha;
                self.cache_conf.historico_mut().adicionar_miss(FonteMiss::Compulsorio);
            }
            // Se não aleatoriamente escolhe o valor a ser substituido e atualiza o registro de miss
            else {
                let posicao:u32 = self.random_source.gen();
                
                self.memoria[indice_procura as usize][(posicao % self.cache_conf.assoc()) as usize] = nova_linha;

                // Se a cache já está cheia é miss de capacidade, senão é de conflito
                if self.cache_cheia {
                    self.cache_conf.historico_mut().adicionar_miss(FonteMiss::Capacidade);
                }
                else {
                    self.cache_conf.historico_mut().adicionar_miss(FonteMiss::Conflito);
                    self.cache_cheia = self.verifica_cache_encheu();
                }
            }

        }
        
        resultado
    }
    fn write(&mut self, valor: i8, endereco:u32) {
        
    }
}
