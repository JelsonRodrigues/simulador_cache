pub enum FonteMiss {
    Capacidade,
    Conflito,
    Compulsorio,
}
pub struct HistoricoAcessos {
    hits:u32,
    misses:u32,

    misses_capacidade:u32,
    misses_conflito:u32,
    misses_compulsorios:u32,
}
impl HistoricoAcessos {
    pub fn new() -> Self { 
        Self { 
            hits:0,
            misses:0, 
            misses_capacidade:0, 
            misses_compulsorios:0, 
            misses_conflito:0 
        } 
    }

    pub fn adicionar_hit(&mut self) {
        self.hits += 1;
    }
    pub fn adicionar_miss(&mut self, tipo_miss:FonteMiss) {
        match tipo_miss {
            FonteMiss::Capacidade => self.misses_capacidade += 1,
            FonteMiss::Conflito => self.misses_conflito += 1,
            FonteMiss::Compulsorio => self.misses_compulsorios += 1,
        }
        self.misses += 1;
    }
    pub fn calcular_taxa_de_hit(&self) -> f32 {
        let total_acessos = self.hits + self.misses;
        if total_acessos > 0 {
            return self.hits as f32 / total_acessos as f32;
        }
        return 0.0;
    }
    pub fn calcular_taxa_de_miss(&self) -> f32 {
        let total_acessos = self.hits + self.misses;
        if total_acessos > 0 {
            return self.misses as f32 / total_acessos as f32;
        }
        return 0.0;
    }
    pub fn calcular_taxa_de_miss_conflito(&self) -> f32 {
        let total_acessos = self.hits + self.misses;
        if total_acessos > 0 {
            return self.misses_conflito as f32 / total_acessos as f32;
        }
        return 0.0;
    }
    pub fn calcular_taxa_de_miss_capacidade(&self) -> f32 {
        let total_acessos = self.hits + self.misses;
        if total_acessos > 0 {
            return self.misses_capacidade as f32 / total_acessos as f32;
        }
        return 0.0;
    }
    pub fn calcular_taxa_de_miss_compulsorio(&self) -> f32 {
        let total_acessos = self.hits + self.misses;
        if total_acessos > 0 {
            return self.misses_compulsorios as f32 / total_acessos as f32;
        }
        return 0.0;
    }
}
