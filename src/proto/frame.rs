use bytes::{Bytes, BytesMut, Buf, BufMut};
use anyhow::{Result, Error};
use tokio_util::codec::{Encoder, Decoder};

// Enum para os tipos de comando, conforme sua especificacoes.
// `repr(u8)` garante que o enum seja armazenado como um unico byte.
#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum CommandType {
    Hello = 0,
    Auth = 1,
    Pub = 2,
    Sub = 3,
    Ack = 4,
    Nack = 5,
    Pull = 6,
    Conf = 7,
    Heartbeat = 8,
    Admin = 9,
}

// Implementa a conversao de u8 para CommandType.
// Isso sera util na hora de decodificar (ler) os bytes.
impl TryFrom<u8> for CommandType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CommandType::Hello),
            1 => Ok(CommandType::Auth),
            2 => Ok(CommandType::Pub),
            3 => Ok(CommandType::Sub),
            4 => Ok(CommandType::Ack),
            5 => Ok(CommandType::Nack),
            6 => Ok(CommandType::Pull),
            7 => Ok(CommandType::Conf),
            8 => Ok(CommandType::Heartbeat),
            9 => Ok(CommandType::Admin),
            _ => Err(anyhow::anyhow!("Tipo de comando invalido: {}", value)),
        }
    }
}

// A estrutura principal do nosso frame, que representa uma mensagem completa
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub command_type: CommandType,
    // Usamos `Bytes` porque e uma fatia de bytes otimizada para ser
    // compartilhada entre threads sem copias. Perfeito para nosso payload.
    pub payload: Bytes,
}

#[allow(dead_code)]
impl Frame {
    /// Construtor simples para PUB
    pub fn new_pub(payload: Bytes) -> Self {
        Frame {
            command_type: CommandType::Pub,
            payload,
        }
    }

    /// Contrutor simples para SUB
    pub fn new_sub(channel: &str) -> Self {
        Frame {
            command_type: CommandType::Sub,
            payload: Bytes::from(channel.to_string()),
        }
    }
}

// O Codec que sera usado pelo Tokio para codificar e decodificar nossos frames.
pub struct MessageCodec;

// O encoder e responsavel por transforma um `Frame` em bytes.
impl Encoder <Frame> for MessageCodec {
    type Error = Error;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result <()> {
        // Formato: | TYPE (1 byte) | LEN (4 bytes) | PAYLOAD (N bytes) |

        let payload_len = item.payload.len() as u32;

        // Reserva espaco no buffer para evitar realocacoes.
        dst.reserve(1 + 4 + payload_len as usize);

        // Escreve o tipo do comando como u8.
        dst.put_u8(item.command_type as u8);
        // Escreve o tamanho do payload como um inteiro de 32 bytes (big-endian).
        dst.put_u32(payload_len);
        //Escreve o payload em si.
        dst.put(item.payload);

        Ok(())
    }
}

// O Decoder é responsável por transformar bytes em um `Frame`.
impl Decoder for MessageCodec {
    type Item = Frame;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        // Precisamos de pelo menos 5 bytes para ler o tipo e o tamanho.
        // 1 (TYPE) + 4 (LEN)
        if src.len() < 5 {
            // Não temos dados suficientes ainda, pedimos para o Tokio esperar mais.
            return Ok(None);
        }

        // Lê o tamanho do payload sem avançar o cursor do buffer.
        let payload_len = (&src[1..5]).get_u32() as usize;

        // Verifica se o frame completo já chegou.
        // Tamanho total = 1 (TYPE) + 4 (LEN) + payload_len
        if src.len() < 5 + payload_len {
            // O frame ainda não chegou por completo. Espera mais dados.
            // Precisamos garantir que o buffer tenha espaço suficiente para o frame completo.
            src.reserve(5 + payload_len - src.len());
            return Ok(None);
        }

        // Se chegamos aqui, temos um frame completo no buffer `src`.
        
        // Avança o cursor do buffer em 1 byte e lê o tipo do comando.
        let command_type_u8 = src.get_u8();
        let command_type = CommandType::try_from(command_type_u8)?;
        
        // Avança o cursor em 4 bytes (já que `get_u32` o consome).
        let _ = src.get_u32(); // Já lemos o tamanho, agora só avançamos.

        // `split_to` é super eficiente: cria uma nova `BytesMut` com `payload_len` bytes
        // do início de `src` sem fazer cópia de memória.
        let payload = src.split_to(payload_len).freeze();

        Ok(Some(Frame { command_type, payload }))
    }
}