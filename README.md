# Improved Lurk

> See my [WIKI](https://github.com/The24Kings/LurkProtocol/wiki) for more in depth information

A new and improved version of my [Lurk Server](https://github.com/The24Kings/lurk-server) written in the Rust programming language!

This rewrite was from the ground up, implementing better data serialization and deserialization. Making it super easy to add new messages to the protocol
or edit existing ones. Simply make a new struct, add it to the types and write the serializer and deserializer methods.

```
 ______    _     _           _____ 
|___  /   | |   | |         / ____|                         
   / / ___| | __| | __ _   | (___   ___ _ ____   _____ _ __ 
  / / / _ \ |/ _` |/ _` |   \___ \ / _ \ '__\ \ / / _ \ '__|
 / /_|  __/ | (_| | (_| |   ____) |  __/ |   \ V /  __/ |   
/_____\___|_|\__,_|\__,_|  |_____/ \___|_|    \_/ \___|_|  

You find yourself standing in front of the gaping maw of a towering tree.
You hear a booming voice from above telling you to enter, but beware for danger lay ahead!

         @@@@@@@@@@@@@@@@@@@@@@@@@@@@
      @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ 
     @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ 
   @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ 
  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  
  @@@@@@@@@@@@@@  '.@@@@@@@@@@@@@@@@@.--.@@@@@@@@@ 
    @@@@@@@@\   @@  ¯ @@@@@@@@@@@ '¯¯ ___..@@@@@@  
     @@@@@@@@|                 @    .'@@@@@@@@@@   
        @@@@@@\                    /@@@@@@@@  
               \                  / 
               |   .--'|__|'--.   |
               |  /.--'/  \'--.\  |
   __  ___     /      /____\      \     ___
 _(  )(   )_  |     .' .''. '.     |  _(   )__  __      __
(           )_|    |__/    \__|    |_(        )(  )_   (
             /                      \__             )_(¯
_______.---./    .'                    \_.--._ ___________
  --''¯        _/    __                       '--..       
             ''    .'
```

> I've semi-learned how lifetimes work, which has made my life easier when implementing the packet stuct

## Packet Example

> One of the simplest packets in the protocol

```RUST
#[derive(Debug, Clone)]
pub struct Start {
    pub author: Option<Arc<TcpStream>>, 
    pub message_type: u8,
}

impl Default for Start {
    fn default() -> Self {
        Start {
            author: None,
            message_type: 6
        }
    }
}

impl<'a> Parser<'a> for Start {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Package into a byte array
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.message_type);
        
        // Send the packet to the author
        writer.write_all(&packet).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write packet to buffer",
            )
        })?;

        debug_packet!(&packet);

        Ok(())
    }
    fn deserialize(packet: Packet) -> Result<Self, std::io::Error> {
        Ok(Start {
            author: packet.author,
            message_type: packet.message_type,
        })
    }
}
```

As you can see, a serialize method packs the struct into a byte stream to be sent off to the connect client.
Deserialize will construct the data into the struct to be used later by the server.
Later this can be called to read and write data to/ from the connected client.

## Client Receive

```RUST
let mut buffer = vec!(0; 32);

let packet = Packet::read_into(self.stream.clone(), packet_type[0], &mut buffer)?;

let object = match Parser::deserialize(packet) {
    Ok(deserialized) => Type::Loot(deserialized),
    Err(e) => {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to deserialize packet: {}", e),
        ));
    }
};

// Send the packet to the sender
Some(object)
```

Once the packet has been successfully deserialized into our data structure, we can send it off via the MPSC channel.

## Hasta La Vista

```RUST
match packet {
    Some(pkt) => {
        self.sender.send(pkt.clone()).map_err(|e| { // If the send fails with SendError, it means the server thread has closed
            std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("Failed to send packet: {}", e),
            )
        })?;

        Ok(pkt)
    },
    None => {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "You tried to send the server a bad packet... naughty!",
        ))
    }
}
```

If for whatever reason this fails, Rust gives us a nice way to handle these errors.

## Error Handling

> Done of the connection thread

```RUST
match client.read() {
    Ok(_) => {
        println!("[CONNECTION] Packet read successfully");
    }
    Err(e) => {
        match e.kind() {
            std::io::ErrorKind::ConnectionReset => {
                eprintln!("[CONNECTION] Connection reset by peer. Terminating thread.");
            }
            std::io::ErrorKind::ConnectionAborted => {
                eprintln!("[CONNECTION] Connection aborted. Terminating thread.");
            }
            std::io::ErrorKind::NotConnected => {
                eprintln!("[CONNECTION] Not connected. Terminating thread.");
            }
            std::io::ErrorKind::BrokenPipe => {
                eprintln!("[CONNECTION] Broken pipe. Terminating thread.");
            }
            std::io::ErrorKind::UnexpectedEof => {
                eprintln!("[CONNECTION] Unexpected EOF. Terminating thread.");
            }
            std::io::ErrorKind::Unsupported => {
                eprintln!("[CONNECTION] Unsupported operation. Terminating thread.");
            }
            _ => {
                eprintln!("[CONNECTION] Non-terminal error: '{}'. Continuing.", e);
                continue; // Continue processing other packets
            }
        }

        // If we reach here, it means the connection was closed
        // Ensure the server thread is notified of the disconnection
        client
            .sender
            .send(Type::Leave(Leave {
                author: Some(stream.clone()),
                ..Leave::default()
            }))
            .unwrap_or_else(|_| {
                eprintln!("[CONNECTION] Failed to send leave packet");
            });

        break;
    }
}    
```

If any other critical errors appear in production, a new ErrorKind will be added to close the connection!