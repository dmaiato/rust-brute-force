use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::mem;
use std::time::Instant;

// Função para gerar uma string aleatória de tamanho fixo
fn gerar_string_aleatoria(tamanho: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(tamanho)
        .map(char::from)
        .collect()
}

// Função principal que realiza a busca pela colisão
fn buscar_colisao(bits: usize) {
    println!("\nIniciando a Busca para Mini-Hash de {} bits!!", bits);

    let tempo_inicio: Instant = Instant::now();
    let mut tentativas: u64 = 0;

    // Dicionário/Tabela Hash para armazenar os hashes
    // A chave será o Mini-Hash (u64 suporta até 64 bits) e o valor a String original
    let mut mapa_hashes: HashMap<u64, String> = HashMap::new();

    loop {
        tentativas += 1;
        let entrada: String = gerar_string_aleatoria(16); // String aleatória de 16 caracteres 

        // Calcula o SHA-256 completo
        let mut hasher = Sha256::new();
        hasher.update(entrada.as_bytes());
        let hash_completo = hasher.finalize();

        // Extrai o Mini-Hash truncando os primeiros bytes [cite: 22, 23]
        let mini_hash: u64 = if bits == 32 {
            // Pega os primeiros 4 bytes (32 bits)
            let mut bytes: [u8; 4] = [0u8; 4];
            bytes.copy_from_slice(&hash_completo[0..4]);
            u32::from_be_bytes(bytes) as u64
        } else if bits == 64 {
            // Pega os primeiros 8 bytes (64 bits)
            let mut bytes: [u8; 8] = [0u8; 8];
            bytes.copy_from_slice(&hash_completo[0..8]);
            u64::from_be_bytes(bytes)
        } else {
            panic!("Tamanho de bits não suportado neste teste.");
        };

        // Verifica a colisão: O processo para assim que encontrar duas strings diferentes com o mesmo Mini-Hash
        if let Some(entrada_anterior) = mapa_hashes.get(&mini_hash) {
            if *entrada_anterior != entrada {
                let tempo_total = tempo_inicio.elapsed();

                // Cálculo aproximado de memória RAM utilizada pela Tabela Hash
                // (Tamanho da Chave + Tamanho do Valor) * Capacidade alocada
                let tamanho_chave = mem::size_of::<u64>();
                let tamanho_valor = mem::size_of::<String>() + 16; // String struct + heap allocation
                let memoria_estimada_bytes =
                    (tamanho_chave + tamanho_valor) * mapa_hashes.capacity();
                let memoria_estimada_mb = memoria_estimada_bytes as f64 / (1024.0 * 1024.0);

                println!("COLISÃO ENCONTRADA após {} tentativas!", tentativas); // [cite: 27]
                println!("Tempo total de execução: {:?}", tempo_total); // [cite: 28]
                println!(
                    "Memória RAM utilizada (aproximada): {:.2} MB",
                    memoria_estimada_mb
                ); // 
                println!("Entrada 1: {}", entrada_anterior);
                println!("Entrada 2: {}", entrada);
                println!("Mini-Hash Comum (Hex): {:x}", mini_hash);

                // Para exibir o hash completo das duas entradas para a prova [cite: 55]
                let mut hasher1 = Sha256::new();
                hasher1.update(entrada_anterior.as_bytes());
                println!("Hash 1 completo: {:x}", hasher1.finalize());

                let mut hasher2 = Sha256::new();
                hasher2.update(entrada.as_bytes());
                println!("Hash 2 completo: {:x}", hasher2.finalize());

                break; // Interrompe o loop 
            }
        }

        // Armazena no Dicionário se não houve colisão
        mapa_hashes.insert(mini_hash, entrada);
    }
}

fn main() {
    // Executa a busca para 32 bits.
    // Para coletar as 5 a 10 execuções pedidas, basta colocar isso num loop for[cite: 26].
    // for i in 1..=5 {
    //     println!("--- Execução {} ---", i);
    //     buscar_colisao(32);
    // }

    // Aviso: A execução de 64 bits demorará MUITO mais tempo e memória.
    buscar_colisao(64);
}
