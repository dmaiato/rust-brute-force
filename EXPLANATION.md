# EXPLANATION.md: Anatomia do Algoritmo de Colisão

Este documento fornece uma explicação técnica detalhada sobre a estrutura, sintaxe e lógica do software desenvolvido em Rust para o desafio do Paradoxo do Aniversário (Atividade Avaliada 1 - TADS).

## 1. Arquitetura de Concorrência e Memória

Para garantir a máxima performance e simular um ambiente de backend de alta segurança, o programa utiliza um modelo de **Memória Compartilhada Segura**.

### Arc (Atomic Reference Counting)
Rust possui um sistema de "propriedade" (*ownership*) que impede que duas threads acessem o mesmo dado simultaneamente para evitar corrupção. 
- **O que faz:** O `Arc` é um ponteiro inteligente que permite que o Mapa de Hashes e os Contadores sejam compartilhados entre várias threads com segurança.
- **Sintaxe:** `Arc::new(...)` encapsula o dado, e `Arc::clone(&dado)` cria uma nova referência para cada thread.

### DashMap (Sharded Hash Map)
Em sistemas concorrentes tradicionais, usar um dicionário comum exigiria uma "trava" (*Mutex*) que pararia o programa toda vez que uma thread fosse escrever algo. 
- **Lógica:** O `DashMap` utiliza **sharding** (fragmentação). Ele divide a tabela interna em vários segmentos independentes.
- **Vantagem:** Isso permite que a Thread A insira um hash em um segmento enquanto a Thread B trabalha em outro, eliminando gargalos de espera e aproveitando 100% da CPU.

---

## 2. Sincronização Atômica

O controle do estado global do programa (número de tentativas e se a colisão foi achada) é feito via tipos **Atômicos**.

- **AtomicU64 (`tentativas_totais`):** Um contador de 64 bits que pode ser atualizado por todas as threads simultaneamente sem risco de erro de contagem.
- **AtomicBool (`encontrou_colisao`):** Funciona como um interruptor global.
- **Ordering (Relaxed):** Indica ao processador que a ordem exata das operações de memória não é crítica, priorizando a velocidade bruta, já que apenas o valor final nos interessa para o relatório.

---

## 3. Lógica de Geração e Hashing

O algoritmo segue um ciclo de vida rigoroso a cada iteração:

1. **Geração Aleatória:** A função `gerar_string_aleatoria` utiliza a biblioteca `rand` para criar sequências alfanuméricas de 16 caracteres.
2. **Cálculo de Hash (SHA-256):** A entrada é processada pelo algoritmo SHA-256 padrão, gerando uma saída de 256 bits.
3. **Truncamento (Mini-Hash):** Para observar a colisão em tempo útil (conforme solicitado no trabalho), o código "corta" o hash original:
   - **32 bits:** Extrai os primeiros 4 bytes.
   - **64 bits:** Extrai os primeiros 8 bytes.
- **Sintaxe:** Usamos `from_be_bytes` para converter esses bytes diretamente em um número inteiro (`u32` ou `u64`), o que é muito mais rápido do que manipular strings hexadecimais na memória.

---

## 4. O Algoritmo de Busca de Colisão

A lógica de detecção é baseada no comportamento do método de inserção:

1. A thread gera um **Mini-Hash**.
2. Ela tenta inseri-lo no `DashMap`.
3. O `DashMap` possui uma propriedade única: se você tenta inserir algo em uma chave que já existe, ele **retorna o valor que estava lá antes**.
4. **Condição de Sucesso:** Se o mapa retornar "Alguma Coisa" (`Some`), significa que aquele Mini-Hash já foi gerado por outra string anteriormente. 
5. **Interrupção:** A thread que detectou a colisão sinaliza o `AtomicBool`. Todas as outras threads verificam esse booleano no início de seu próximo loop e encerram a execução imediatamente.

---

## 5. Complexidade e Escalabilidade

O programa demonstra visualmente o **Paradoxo do Aniversário**:
- Para **32 bits**, o espaço é de $2^{32}$, mas a colisão é encontrada com aproximadamente $\sqrt{2^{32}} \approx 65.536$ tentativas.
- O uso de **Rust** permite que o gerenciamento de memória (subida e descida no gráfico do sistema) seja automático e eficiente, liberando a RAM assim que cada execução termina (`Drop` de escopo).

---