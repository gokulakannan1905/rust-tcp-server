use colored::Colorize;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::{collections::HashMap, fs};
pub fn read_from_stream(stream: &TcpStream) -> Result<String, std::io::Error> {
    let buf_reader = BufReader::new(stream);
    let mut received_data = String::new();
    // let received_data = buf_reader
    //                                     .lines()
    //                                     .map(|line| line.unwrap())
    //                                     .take_while(|line| !line.is_empty())
    //                                     .collect::<Vec<_>>().join("\n");

    for line in buf_reader.lines() {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    break;
                }
                received_data.push_str(&line);
                received_data.push_str("\n");
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    // println!("{}", received_data);
    Ok(received_data)
}

pub fn send_to_steam(stream: &mut TcpStream, msg: &str) -> Result<(), std::io::Error> {
    let data = msg.to_string() + "\n\n";
    stream.write_all(data.as_bytes())
}

pub fn authenticate(
    stream: &mut TcpStream,
    user: &str,
    pass: &str,
) -> Result<bool, std::io::Error> {
    // Reading the users file and storing in a string
    let contents = match fs::read_to_string("users") {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Problem reading file: {:?}", e.to_string());
            return Err(e);
        }
    };
    
    // Parsing the users file and storing in a HashMap
    let mut users = HashMap::new();
    for line in contents.lines() {
        let mut parts = line.split(' ');
        let name = parts.next().unwrap();
        let pass = parts.next().unwrap();
        users.insert(name.to_string(), pass.to_string());
    }
    // dbg!(user, pass);    

    // Checking if the user and password are valid
    if users.contains_key(user) && users[user] == pass {
        send_to_steam(stream, "OK")?;
        Ok(true)
    } else {
        send_to_steam(stream, "Invalid username or password")?;
        Ok(false)
    }
}

pub fn list_directory(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let contents = match fs::read_dir("./data") {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Problem reading directory: {:?}", e.to_string());
            return send_to_steam(stream, "Problem reading directory");
        }
    };
    let mut files = Vec::new();
    for entry in contents {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        if metadata.is_file() {
            files.push(entry.file_name().into_string().unwrap());
        }
    }
    send_to_steam(stream, &files.join("\n"))
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            return Err(e);
        }
    };
    Ok(contents)
}

fn write_file(path: &str, contents: &str) -> Result<(), std::io::Error> {
    fs::write(path, contents)
}

pub fn send_file(stream: &mut TcpStream, filename: &str) -> Result<(), std::io::Error> {
    let path = format!("./data/{}", filename);
    let contents = match read_file(path.as_str()) {
        Ok(contents) => contents,
        Err(_) => {
            return send_to_steam(stream, "File not found");
        }
    };
    let mut data = Vec::new();
    data.push(format!("File: {}", filename.to_string().red()));
    let mut length = 1;
    for line in contents.lines() {
        data.push(format!("{}\t{}", length.to_string().yellow(), line));
        length += 1;
    }
    send_to_steam(stream, &data.join("\n"))
}

pub fn edit_file(
    stream: &mut TcpStream,
    filename: &str,
    line_number: usize,
) -> Result<(), std::io::Error> {
    if line_number == 0 {
        return send_to_steam(stream, "Line number must be greater than 0");
    }

    let path = format!("./data/{}", filename);
    let contents = match read_file(path.as_str()) {
        Ok(contents) => contents,
        Err(_) => {
            return send_to_steam(stream, "File not found");
        }
    };
    let mut vec: Vec<String> = Vec::new();
    for line in contents.lines() {
        vec.push(line.to_string());
    }
    let mut space_count:u8 = 0;
    if line_number > vec.len() {
        return send_to_steam(stream, "Line number out of bounds");
    } else {
        for c in vec[line_number-1].chars(){
            if c == ' '{
                space_count +=1;
            }else{
                break;
            }
        }
        send_to_steam(stream, vec[line_number - 1].as_str().trim())?;
    }
    let data = read_from_stream(stream)?;
    if !data.is_empty() {
        vec[line_number-1].clear();
        for _ in 0..space_count{
            vec[line_number-1].push(' ');
        }
        vec[line_number - 1].push_str(data.trim());
        write_file(&path, &vec.join("\n"))?;
    }
    send_to_steam(stream, "Successfully Edited")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_file() {
        let contents = read_file("./data/hello.txt").unwrap();
        assert_eq!(contents, "hello\n");
    }
    #[test]
    fn test_read_file_fail() {
        let contents = match read_file("./data/awhdkbkesf.txt") {
            Ok(contents) => contents,
            Err(e) => {
                return assert_eq!(e.to_string(), "No such file or directory (os error 2)");
            }
        };
        assert_eq!(contents, "hello\n");
    }
}
