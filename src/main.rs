use dashmap::DashMap; // Tabela Hash thread-safe (concorrente)
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng}; // Geração de números/strings aleatórias
use sha2::{Digest, Sha256}; // Crate para criptografia SHA-256
use std::mem; // Verificação de tamanho de tipos na memória
use std::sync::Arc; // Ponteiro de referência compartilhada entre threads
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering}; // Sincronização atômica
use std::thread; // Gerenciamento de Threads
use std::time::Instant; // Medição de tempo de execução

/// Gera uma string aleatória de caracteres alfanuméricos
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

    // Detecta quantos núcleos (threads lógicas) o computador possui
    let num_threads = num_cpus::get();
    println!("Iniciando {} threads trabalhadoras...", num_threads);

    // Vetor de pares
    let mut handles = vec![];

    for _ in 0..num_threads {
        // Clona os ponteiros (Arc) para cada thread
        let mapa_clone = Arc::clone(&mapa_hashes);
        let tentativas_clone = Arc::clone(&tentativas_totais);
        let encontrou_clone = Arc::clone(&encontrou_colisao);

        let handle = thread::spawn(move || {
            // O processo para assim que o algoritmo encontrar duas strings diferentes com o mesmo Mini-Hash
            // A thread verifica constantemente se alguma outra thread já achou a colisão
            while !encontrou_clone.load(Ordering::Relaxed) {
                // Incrementa o contador global de tentativas
                tentativas_clone.fetch_add(1, Ordering::Relaxed);

                let entrada = gerar_string_aleatoria(16);

                // Processo de Hashing
                let mut hasher = Sha256::new();
                hasher.update(entrada.as_bytes());
                let hash_completo = hasher.finalize();

                // Truncamento: Extrai apenas os bits necessários (32 ou 64)
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

                // Tenta inserir no mapa. Se o 'insert' retornar algo, é porque a chave já existia. (Some)
                if let Some(entrada_anterior) = mapa_clone.insert(mini_hash, entrada.clone()) {
                    // Verifica se não é a mesma string (improvável, mas boa prática)
                    if entrada_anterior != entrada {
                        // Sinaliza para todas as outras threads pararem
                        encontrou_clone.store(true, Ordering::Relaxed);

                        let tempo_total = tempo_inicio.elapsed(); // Registra o tempo total de execução

                        // Calcula a quantidade de memória RAM utilizada (aproximada)
                        // Multiplicamos pela capacidade atual do DashMap
                        // (tamanho u64 + tamanho String) * capacidade alocada
                        let tamanho_chave = mem::size_of::<u64>();
                        let tamanho_valor = mem::size_of::<String>() + 16;
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
