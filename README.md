Trying to make a bittorrent client using Rust. 

Reading: 
- https://en.wikipedia.org/wiki/Bencode
- https://www.bittorrent.com
- https://en.wikipedia.org/wiki/BitTorrent
- https://www.youtube.com/watch?v=vpSkBV5vydg - to understand UTF-8. Needed to understand why we do bencoding.
- https://wiki.theory.org/BitTorrentSpecification#Identification - formal guide
- https://arxiv.org/pdf/1311.1195#:~:text=Peer%20Selection%20Strategy&text=Different%20mechanisms%20are%20used%20in,sets%20in%20BitTorrent%20%5B52%5D.
- https://www.beautifulcode.co/blog/58-understanding-bittorrent-protocol
- https://doc.rust-lang.org/book/ch21-01-single-threaded.html Single threaded socket programming in Rust
Tutorial: 

what is NOT being done

Magnet links
DHT
bitorrent V2 - Fixes the broken SHA-1 hash
announce-list
url-list
Merkle trees
Multi-file download 
 
Bitorrent client

1) Bencoding
This format specification for encoding and decoding binary data in a structured way.  
All communication through the network happens on messages that are bencoded and then decoded.

for the rules: Wikipedia: https://en.wikipedia.org/wiki/Bencode
For a (very bad) Rust implementation: https://github.com/Dragonado/corrent/blob/main/src/bencode.rs

My current implementation of the decoder is O(N^2) where N is size of input bytes. But it should be possible to do it in O(N). My code will perform especially bad the more popular the file is because the tracker will return a list of a million peers. 

2) Reading a sample torrent file
 Download any (legal) torrent and inspect its contents. It will bencoded so obviously it wont be human readable. Run your decoder on the torrent file and then read the structured data.

https://en.wikipedia.org/wiki/Torrent_file

These are the important fields:
1. announce: Single tracker that has the information of the peers.
2. Info:
    1. Length: size of the final file in bytes. 
    2. name: optional suggestion for what name to save the file as.
    3. piece-length: The final final is split into several equally sized pieces. This value is the size of each piece. Although the last piece of the final can have any value between [1, piece_length].
    4. pieces: The array of hashes for every piece. The order is important here. So size of pieces array is equal to the number of pieces the file is split into.

3) Communicating with the tracker.

Make a simple GET request to the tracker asking for peers to connect to.

Important fields:
1. Info hash: This is a SHA1 hash of the “value” of the “info” dictionary that we got from the tracker. This uniquely identifies the file that we want to download.
2. peer_id: This is just a random array of 20 bytes that uniquely defines us as a peer. I just put the string “Dragonado in the goat” into SHA1 and got a 20 byte hash.
3. port: this is the port id that the tracker is listening to. We try all ids from 6881 to 6889. If none of them work then try another tracker or wait for some time.

We then get a list of peers in the network that have our file.

Link: https://github.com/Dragonado/corrent/commit/3ed12a3e7344af7a65a6a161021a82a418df8cea

4) Communicating with other peers.
All peer in the network are equivalent. 

For arguments sake, I will refer to myself as the downloader and another peer as the uploader.

As discussed above, I get the IP address and port of the uploader from the tracker.

Every peer has 2 bits of info denoting 4 states:
1. Choked or not choked.
2. Interested or not interested 

The Downloader is Interested in the Uploader. (The Downloader sends an "Interested" message to the Uploader, saying, "You have pieces that I want.")
The Uploader has Unchoked the Downloader. (The Uploader sends an "Unchoke" message to the Downloader, saying, "I am willing to send you data if you request it.”)

Let us select a random peer from the peer list we got from the tracker and download the file from them.

To download a file we must:
1. Handshake with the uploader - ie establish connection.
    1. This is done by sending a special request.
2. Keep the connection alive until all your pieces are downloaded.

Not covering: 
- Piece selection algorithm. There are some algorithms out there like FindRarestPiece first or random piece but these are optimizations and not essential.
- multi-threading to download multiple pieces concurrently. I currently go piece by piece.
- Trying different peers if a particular peer is not good. 
- Its possible that a peer will only have half the file but in my implementation I will continuously query them for the whole file. Its probably good practice to give up on the connection if a particular peer does not have the piece after we asked them 10 times but I can’t be bothered with that implementation.
- Choose peers and random and download random pieces of your file. This ensures no particular peer is biased towards any file piece. This gives good performance.
- Consider the scenario of 100 peers where only 1 peer has the file. If every peer requests the first 99% of the data in sequential order and before they can get the 1% this particular peer goes offline then no one has the data. Instead of every peer asked for a random piece then very quickly for every piece there will be atleast 2 or more people that have it. 


More on choking algorithms: https://medium.com/@abhinavcv007/bittorrent-part-1-the-engineering-behind-the-bittorrent-protocol-04e70ee01d58

5) Downloading your file.
lorem ipsum
6) Uploading your file for others to download

Previously we have implemented all the downloading functionality. Now we have to implement the uploading functionality so that others can downloaded the pieces that we have.

This is important because:
1. (philosophical reason) We need to give back to the system when we benefit from it. This keeps the system going or else if everyone was selfish then the system would very quickly fail. This is also the reason why many alumni donate lot of money their universities because its because of the university that they have so much money.
2. (technical reason) Its possible that all other peers identify that you are not uploading anything and choke you out from downloading anything from anyone.  

fun fact: It’s also possible that the peer that we were downloading from, asks us if they can download some other piece from us [Insert umbrella academy meme]. 

// …
// …
After all this,  we announce to the tracker that we are also a peer in the network and ready to upload files for others. The tracker then adds our ip into their torrent file.

Not covering:- All connections so far are HTTP which is implemented using TCP. But you can also announce yourself to the tracker using UDP which is more efficient for the tracker8.
