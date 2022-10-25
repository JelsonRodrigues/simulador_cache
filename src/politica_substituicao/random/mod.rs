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
    random_source:ThreadRng,
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
            random_source 
        } 
    }
    
    // Retorna uma referencia de read-only do historico de acessos da cache
    pub fn historico_acessos(&self) -> &HistoricoAcessos {
        self.cache_conf.historico()
    }
    
    fn trata_falta_leitura(&mut self, endereco:u32) {
        /*
        Atualmente eu crio uma nova linha, inicializo os valores da linha de acordo com
        a memoria de nivel inferior e coloco esta linha na posicao correta da cache,
        so que nao estou liberando a memoria que a linha anterior ocupava, sera o rust
        ja faz isso?

        A forma mais correta e primeiro eu identificar o local de onde deve ficar o bloco
        e ja ler do nivel inferior diretamente para la, assim nao tenho que me preocupar com 
        desalocacao, so tem que criar metodos na Linha para que possa ser alterado os valores
        de tag validade e o vetor com os bytes
        */
        
        
        // Cria a nova linha que entrara na cache
        let mut nova_linha = Linha::new(&self.cache_conf);
        nova_linha.tag = self.cache_conf.tag_do_endereco(endereco);
        nova_linha.validade = true;

        // 
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
    /*
    Esta funcao procura por algum bloco vazio na linhas, e retorna uma tupla
    que contem uma flag true/false se encontrou ou nao e caso seja true, 
    o segundo elemento da tupla e o indice da coluna da matriz, onde esta vazio
    */
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


        /*
        Nesta parte, de verdade todos os conjuntos da associatividade deveriam 
        ser verificados ao mesmo tempo
        Para implementar em codigo talvez desse para fazer utilizando threads,
        deste modo teria um comportamento proximo ao real, apenas o tamanho da associatividade
        teria que ser sempre <= ao numero de threads disponiveis no computador

        No codigo atual os conjuntos sao verificados de forma sequencial, por facilidade
        de implementacao
        */
        for item in &self.memoria[indice_procura as usize] {
            if item.tag == tag_procura && item.validade {
                self.cache_conf.historico_mut().adicionar_hit();
                // Se endereçado a byte, retorna um único byte
                // Senão retorna o bloco o numero de bytes correspondente ao endereco?
                // Nao da pra retornar o bloco inteiro, porque podem ter varios blocos dentro 
                // de um bloco da cache, bsize > 1
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

        // Occoreu um miss
        if resultado.len() == 0 {
            
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
