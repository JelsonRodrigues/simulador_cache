use crate::memoria::Memoria;
use crate::historico_acessos::HistoricoAcessos;

pub struct Cache {
    historico:HistoricoAcessos,
    
    nsets:u32,
    bsize:u32,
    assoc:u32,

    tag_size:u8,
    index_size:u8,
    offset_size:u8,
    address_size:u8,
    bytes_por_palavra:u8,
    byte_addressed:bool,

    nivel_inferior:Box<dyn Memoria>,
}

impl Cache {
    pub fn new(nsets: u32, bsize: u32, assoc: u32, address_size: u8, byte_addressed: bool, bytes_por_palavra:u8, nivel_inferior: Box<dyn Memoria>) -> Self {
        if self::Cache::verifica_potencia_2(nsets) == false {panic!("O VALOR DE NSETS DEVE SER POTÊNCIA DE 2")}
        if self::Cache::verifica_potencia_2(bsize) == false {panic!("O VALOR DE BSIZE DEVE SER POTÊNCIA DE 2")}
        if self::Cache::verifica_potencia_2(assoc) == false {panic!("O VALOR DE ASSOC DEVE SER POTÊNCIA DE 2")}
        if self::Cache::verifica_potencia_2(address_size as u32) == false {panic!("O VALOR DE ADDRESS_SIZE DEVE SER POTÊNCIA DE 2")}
        if self::Cache::verifica_potencia_2(bytes_por_palavra as u32) == false {panic!("O VALOR DE BYTES_POR_PALAVRA DEVE SER POTÊNCIA DE 2")}
        if bsize < bytes_por_palavra as u32 {panic!("O NÚMERO DE BYTES DENTRO DO BLOCO (BSIZE) DEVE SER NO MÍNIMO O TAMANHO DE UMA PALAVRA DE MEMÓRIA")}

        let historico = HistoricoAcessos::new();
        let offset_size;
        let tag_size;
        let index_size;

        if byte_addressed {
            offset_size = f32::log2(bsize as f32) as u8;
        }
        else {
            let palavras_por_bloco = bsize as f32 / bytes_por_palavra as f32;
            offset_size = f32::log2(palavras_por_bloco) as u8;
        }

        index_size = f32::log2(nsets as f32) as u8;
        tag_size = address_size - index_size - offset_size;

        Self { 
            historico, 
            nsets, 
            bsize, 
            assoc, 
            tag_size, 
            index_size, 
            offset_size, 
            address_size, 
            bytes_por_palavra,
            byte_addressed, 
            nivel_inferior 
        } 
    }

    pub fn verifica_potencia_2(numero:u32) -> bool {
        f32::log2(numero as f32).fract() == 0.0
    }

    pub fn nsets(&self) -> u32 { self.nsets }

    pub fn historico(&self) -> &HistoricoAcessos { &self.historico }

    pub fn historico_mut(&mut self) -> &mut HistoricoAcessos { &mut self.historico }

    pub fn bsize(&self) -> u32 { self.bsize }

    pub fn assoc(&self) -> u32 { self.assoc }

    pub fn tag_size(&self) -> u8 { self.tag_size }

    pub fn index_size(&self) -> u8 { self.index_size }

    pub fn offset_size(&self) -> u8 { self.offset_size }

    pub fn address_size(&self) -> u8 { self.address_size }

    pub fn bytes_por_palavra(&self) -> u8 { self.bytes_por_palavra }

    pub fn enderecado_byte(&self) -> bool { self.byte_addressed }

    pub fn nivel_inferior(&self) -> &dyn Memoria { self.nivel_inferior.as_ref() }

    pub fn nivel_inferior_mut(&mut self) -> &mut dyn Memoria { self.nivel_inferior.as_mut() }

    pub fn tag_do_endereco(&self, endereco:u32) -> u32 {
        endereco >> (self.index_size + self.offset_size)
    }

    pub fn indice_do_endereco(&self, endereco:u32) -> u32 {
        // Remove os bits do tag
        let indice = endereco << self.tag_size;

        // Desloca para a direita no número de bits do offset e do tag, deixando apenas os bits do índice
        indice >> (self.offset_size + self.tag_size)
    }
    
    pub fn offset_endereco(&self, endereco:u32) -> u32 {
        if self.offset_size == 0 { return 0; }
        
        // Deixa somente os bits do offset
        let resultado = endereco << (self.index_size + self.tag_size);
        
        // Desloca o offset para a direita, e retorna
        resultado >> (self.index_size + self.tag_size)
    }
}