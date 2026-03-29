// Brute-force de colisão de prefixos de hash (mini-hash) usando SHA-256.
//
// O programa gera entradas aleatórias e calcula o SHA-256 de cada uma. A cada iteração
// ele extrai um prefixo de `bits` do hash (o "mini-hash") e guarda este prefixo em um
// HashMap para detectar quando o mesmo prefixo aparece duas vezes (colisão).
//
// Isso demonstra empiricamente o "paradoxo do aniversário": a probabilidade de colisão
// cresce rápido, ficando em torno de 2^(bits/2) tentativas para achar um par.

use memory_stats::memory_stats;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Instant;

/// Dados gerados quando uma colisão de mini-hash é encontrada.
///
/// Usado apenas para formatação de saída; guarda as duas entradas que colidiram,
/// seus hashes completos e métricas de tempo/memória.
struct ColisaoResult {
    bits: usize,   // Quantidade de bits iguais no prefixo do hash
    attempts: u64, // Total de iterações realizadas até encontrar a colisão
    duration: std::time::Duration,
    initial_memory: u64, // Memória física (RSS) no início (em bytes)
    final_memory: u64,   // Memória física (RSS) no momento da colisão
    string1: String,     // Input A (em formato hexadecimal)
    string2: String,     // Input B (em formato hexadecimal)
    hash1: String,       // Hash SHA-256 completo do Input A
    hash2: String,       // Hash SHA-256 completo do Input B
    mini_hash: u64,      // O prefixo de bits comum a ambos (representado em u64)
}

/// Obtém a memória física atual (Resident Set Size) usada pelo processo.
///
/// Usa `memory_stats` para ler o RSS. Se a coleta falhar por qualquer motivo, retorna 0.
/// Isso é útil para monitorar o crescimento do HashMap à medida que geramos muitas entradas.
fn get_allocated_memory() -> u64 {
    memory_stats().map_or(0, |usage| usage.physical_mem as u64)
}

/// Imprime no terminal um relatório detalhado da colisão encontrada.
///
/// O mini-hash (prefixo de `bits`) é exibido em hexadecimal com largura fixa
/// para facilitar a leitura e comparação.
fn imprimir_resultado(r: &ColisaoResult) {
    // Cada caractere hex representa 4 bits. Por exemplo:
    // - 32 bits -> 8 caracteres (8 * 4 = 32)
    // - 48 bits -> 12 caracteres (12 * 4 = 48)
    let hex_chars = r.bits / 4;

    println!("\n========================================");
    println!("COLISÃO ENCONTRADA EM {} BITS ({} HEX)", r.bits, hex_chars);
    println!(
        "Fatia Alvo (Mini-Hash): {:0>width$x}",
        r.mini_hash,
        width = hex_chars
    );
    println!("----------------------------------------");
    println!("Entrada A: {}", r.string1);
    println!("Entrada B: {}", r.string2);
    println!("Hash A:    {}", r.hash1);
    println!("Hash B:    {}", r.hash2);
    println!("----------------------------------------");
    println!("Tentativas:         {}", r.attempts);
    println!("Tempo de execução:  {:?}", r.duration);
    println!("Memória Inicial:    {} MB", r.initial_memory / 1024 / 1024);
    println!("Memória Final:      {} MB", r.final_memory / 1024 / 1024);
    println!("========================================\n");
}

/// Busca exaustivamente um par de entradas diferentes cujos hashes tenham o mesmo
/// prefixo de `bits` (mini-hash).
///
/// Para cada entrada gerada:
/// 1) calcula SHA-256;
/// 2) extrai os `bits` mais significativos do hash (mini-hash);
/// 3) verifica se esse mini-hash já apareceu antes.
///
/// Devido ao Paradoxo do Aniversário, a chance de encontrar uma colisão é alta após
/// cerca de 2^(bits/2) entradas (para `bits` razoáveis). O HashMap permite detectar a
/// colisão em O(1) por verificação de existência de chave.
fn buscar_colisao(bits: usize) {
    // Limite técnico: estamos armazenando o mini-hash em um u64.
    // Para extrair os `bits` mais significativos, precisamos poder deslocar o valor
    // dentro de 64 bits. Por isso, `bits` não pode ser maior que 64.
    if bits > 64 {
        panic!("Este exemplo suporta no máximo 64 bits para o mini-hash.");
    }

    println!("Iniciando busca por colisão de {} bits...", bits);

    // O HashMap guarda o prefixo (mini-hash) como chave para busca em O(1).
    // O valor armazenado é a string (em hex) que gerou aquele prefixo.
    // A capacidade inicial de 1_000_000 é apenas um ponto de partida para reduzir
    // realocações no início da execução.
    let mut hashes: HashMap<u64, String> = HashMap::with_capacity(1_000_000);

    let memoria_inicial = get_allocated_memory();
    let tempo_inicio = Instant::now();
    let mut tentativas: u64 = 0;

    loop {
        tentativas += 1;

        // Gera uma entrada aleatória (16 bytes) e converte para hex.
        // A string hex é usada como input para SHA-256.
        let mut buf = [0u8; 16];
        getrandom::getrandom(&mut buf).expect("falha ao gerar bytes aleatórios");
        let entrada = hex::encode(buf);

        // Calcula o hash SHA-256 da entrada.
        // O resultado tem 32 bytes (256 bits) e usamos apenas o prefixo.
        let mut hasher = Sha256::new();
        hasher.update(entrada.as_bytes());
        let hash_atual = hasher.finalize();

        // Extração dos bits do mini-hash
        // Pegamos os primeiros 8 bytes (64 bits) do hash.
        // Aplicamos um bit-shift para a direita (>>) para manter apenas os bits mais significativos.
        let mut bytes_temp = [0u8; 8];
        bytes_temp.copy_from_slice(&hash_atual[..8]);
        let mini_hash = u64::from_be_bytes(bytes_temp) >> (64 - bits);

        // Verificação de colisão por prefixo (mini-hash).
        //
        // Se já vimos esse prefixo antes, o HashMap retorna a primeira string que
        // gerou o mesmo mini-hash. Como o mini-hash olha apenas para alguns bits do
        // hash, ainda precisamos garantir que as strings sejam diferentes.
        if let Some(entrada_anterior) = hashes.get(&mini_hash) {
            // Se a mesma string foi gerada novamente, não é uma colisão útil.
            if entrada_anterior != &entrada {
                // Recalcula o hash da primeira string apenas para imprimir o resultado.
                // Mantemos os hashes completos para validação e comparação.
                let mut h1 = Sha256::new();
                h1.update(entrada_anterior.as_bytes());

                let resultado = ColisaoResult {
                    bits,
                    attempts: tentativas,
                    duration: tempo_inicio.elapsed(),
                    initial_memory: memoria_inicial,
                    final_memory: get_allocated_memory(),
                    string1: entrada_anterior.clone(),
                    string2: entrada,
                    hash1: hex::encode(h1.finalize()),
                    hash2: hex::encode(hash_atual),
                    mini_hash,
                };

                imprimir_resultado(&resultado);
                return;
            }
        }

        // Insere o mini-hash e a string correspondente no mapa para checagens futuras.
        // Posteriormente, se o mesmo mini-hash aparecer novamente, podemos identificar a colisão.
        hashes.insert(mini_hash, entrada);

        // Log visual de progresso para que o usuário saiba que o programa continua rodando.
        // A cada milhão de tentativas mostramos quantos milhões já foram processados e
        // quanto de RAM o processo está usando (RSS).
        if tentativas % 1_000_000 == 0 {
            println!(
                "... {} milhões de tentativas processadas (RAM: {} MB)",
                tentativas / 1_000_000,
                get_allocated_memory() / 1024 / 1024
            );
        }
    }
}

fn main() {
    // Aqui escolhemos quantos bits do prefixo do hash serão usados para detectar
    // colisões. Valores menores funcionam rápido, mas dão menos confiança (poucos bits).
    // Valores maiores exigem muito mais tempo e memória (como em ataques reais).
    //
    // Para testar outros valores, basta alterar este número ou parametrizar via args.
    buscar_colisao(48);
}
