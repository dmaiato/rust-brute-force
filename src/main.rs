use dashmap::DashMap;
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use sha2::{Digest, Sha256};
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Instant;

fn gerar_string_aleatoria(tamanho: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(tamanho)
        .map(char::from)
        .collect()
}

fn buscar_colisao_concorrente(bits: usize) {
    println!(
        "\nIniciando a Busca Concorrente para Mini-Hash de {} bits!!",
        bits
    );

    let tempo_inicio = Instant::now();

    // Arc (Atomic Reference Counting) permite que múltiplas threads compartilhem a mesma referência de memória com segurança.
    let mapa_hashes = Arc::new(DashMap::new());
    let tentativas_totais = Arc::new(AtomicU64::new(0));
    let encontrou_colisao = Arc::new(AtomicBool::new(false));

    let num_threads = num_cpus::get(); // Descobre quantos núcleos lógicos sua CPU possui
    println!("Iniciando {} threads trabalhadoras...", num_threads);

    let mut handles = vec![];

    for _ in 0..num_threads {
        let mapa_clone = Arc::clone(&mapa_hashes);
        let tentativas_clone = Arc::clone(&tentativas_totais);
        let encontrou_clone = Arc::clone(&encontrou_colisao);

        let handle = thread::spawn(move || {
            // O processo para assim que o algoritmo encontrar duas strings diferentes com o mesmo Mini-Hash
            // A thread verifica constantemente se alguma outra thread já achou a colisão
            while !encontrou_clone.load(Ordering::Relaxed) {
                // Registra o número total de tentativas [cite: 27]
                tentativas_clone.fetch_add(1, Ordering::Relaxed);

                let entrada = gerar_string_aleatoria(16);

                let mut hasher = Sha256::new();
                hasher.update(entrada.as_bytes());
                let hash_completo = hasher.finalize();

                // O algoritmo deve extrair apenas os primeiros N bits (32 e 64 bits) [cite: 23]
                let mini_hash: u64 = if bits == 32 {
                    let mut bytes = [0u8; 4];
                    bytes.copy_from_slice(&hash_completo[0..4]);
                    u32::from_be_bytes(bytes) as u64
                } else if bits == 64 {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&hash_completo[0..8]);
                    u64::from_be_bytes(bytes)
                } else {
                    panic!("Tamanho de bits não suportado.");
                };

                // O DashMap retorna o valor antigo se a chave já existia.
                // Se isso acontecer, achamos nossa colisão!
                if let Some(entrada_anterior) = mapa_clone.insert(mini_hash, entrada.clone()) {
                    if entrada_anterior != entrada {
                        // Sinaliza para todas as outras threads pararem
                        encontrou_clone.store(true, Ordering::Relaxed);

                        let tempo_total = tempo_inicio.elapsed(); // Registra o tempo total de execução [cite: 28]

                        // Calcula a quantidade de memória RAM utilizada (aproximada) [cite: 29]
                        let tamanho_chave = mem::size_of::<u64>();
                        let tamanho_valor = mem::size_of::<String>() + 16;
                        // Multiplicamos pela capacidade atual do DashMap
                        let memoria_estimada_bytes =
                            (tamanho_chave + tamanho_valor) * mapa_clone.capacity();
                        let memoria_estimada_mb = memoria_estimada_bytes as f64 / (1024.0 * 1024.0);

                        println!("\n*** COLISÃO ENCONTRADA! ***");
                        println!(
                            "Tentativas totais globais: {}",
                            tentativas_clone.load(Ordering::Relaxed)
                        );
                        println!("Tempo total de execução: {:?}", tempo_total);
                        println!(
                            "Memória RAM utilizada (aproximada): {:.2} MB",
                            memoria_estimada_mb
                        );
                        println!("Entrada 1: {}", entrada_anterior);
                        println!("Entrada 2: {}", entrada);
                        println!("Mini-Hash Comum (Hex): {:x}", mini_hash);
                        break;
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Aguarda todas as threads finalizarem o trabalho
    for handle in handles {
        handle.join().unwrap();
    }
}

fn main() {
    // for i in 1..=5 {
    //     println!("\n--- Execução {} ---", i);
    //     buscar_colisao_concorrente(32);
    // }

    // Aviso: Para 64 bits, certifique-se de ter bastante RAM livre.
    buscar_colisao_concorrente(64);
}
