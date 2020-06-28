/**
 * NetworkClient constructor.
 * NetworkClient handles network connection and data transfer.
 * NetworkClient is an abstract class.
 * Set local media streaming to params.stream if you wanna send it to remote.
 * Concrete class is assumed WebRTC/Firebase/WebSocket based client.
 * @param {object} params - instanciate parameters (optional)
 */
class NetworkClient {
  constructor(params={}) {
    this.id = params.id !== undefined ? params.id : '';
    this.hostId = '';
    this.roomId = '';

    // connections
    // connection is a object which has a remote peer id in .peer property.
    // a connection per a connected remote peer.
    this.connections = [];
    this.connectionTable = {};  // remote peer id -> connection

    // event listeners
    this.onOpens = [];
    this.onCloses = [];
    this.onErrors = [];
    this.onJoins = [];
    this.onRemoteJoins = [];
    this.onRemoteLeaves = [];
    this.onConnects = [];
    this.onDisconnects = [];
    this.onReceives = [];
    this.onRemoteStreams = [];

    if (params.onOpen !== undefined) this.addEventListener('open', params.onOpen);
    if (params.onClose !== undefined) this.addEventListener('close', params.onClose);
    if (params.onError !== undefined) this.addEventListener('error', params.onError);
    if (params.onJoin !== undefined) this.addEventListener('join', params.onJoin);
    if (params.onRemoteJoin !== undefined) this.addEventListener('remote_join', params.onRemoteJoin);
    if (params.onRemoteLeave !== undefined) this.addEventListener('remote_leave', params.onRemoteLeave);
    if (params.onConnect !== undefined) this.addEventListener('connect', params.onConnect);
    if (params.onDisconnect !== undefined) this.addEventListener('disconnect', params.onDisconnect);
    if (params.onReceive !== undefined) this.addEventListener('receive', params.onReceive);
    if (params.onRemoteStream !== undefined) this.addEventListener('remote_stream', params.onRemoteStream);
  }

  // public
  /**
   * Adds EventListener. Callback function will be invoked when
   * 'open': a connection is established with a signaling server
   * 'close': a connection is disconnected from a signaling server
   * 'error': network related error occurs
   * 'join': joins a room
   * 'remote_join': a remote peer joins a room
   * 'remote_leave': a remote peer leaves a room
   * 'connect': a connection is established with a remote peer
   * 'disconnect': a connection is disconnected from a remote peer
   * 'receive': receives remote data sent from a remote peer
   * 'remote_stream': receives a remote media stream
   *
   * Arguments for callback functions are
   * 'open': {string} local peer id
   * 'close': {string} local peer id
   * 'error': {string} error message
   * 'join': {string} host peer id
   * 'remote_join': {string} remote peer id
   * 'remote_leave': {string} remote peer id
   * 'connect': {string} remote peer id
   * 'disconnect': {string} remote peer id
   * 'receive': {object} component object sent from remote peer
   * 'remote_stream': {MediaStream} remote media stream
   *
   * @param {string} type - event type
   * @param {function} func - callback function
   */
  addEventListener(type, func) {
    switch (type) {
      case 'open':
        this.onOpens.push(func);
        break;
      case 'close':
        this.onCloses.push(func);
        break;
      case 'error':
        this.onErrors.push(func)
        break;
      case 'join':
        this.onJoins.push(func)
        break;
      case 'remote_join':
        this.onRemoteJoins.push(func)
        break;
      case 'remote_leave':
        this.onRemoteLeaves.push(func)
        break;
      case 'connect':
        this.onConnects.push(func)
        break;
      case 'disconnect':
        this.onDisconnects.push(func);
        break;
      case 'receive':
        this.onReceives.push(func);
        break;
      case 'remote_stream':
        this.onRemoteStreams.push(func);
        break;
      default:
        console.log('NetworkClient.addEventListener: Unknown type ' + type);
        break;
    }
  }

  /**
   * Joins a room
   * A child class must override this method.
   * @param {string} id - room id
   */
  join(id) {}

  /**
   * Connects a remote peer
   * A child class must override this method.
   * @param {string} id - remote peer id
   */
  connect(id) {}

  /**
   * Sends data to a remote peer.
   * @param {string} id - remote peer id
   * @param {anything} data
   */
  send(id, data) {}

  /**
   * Broadcasts data to all connected peers.
   * @param {anything} data
   */
  broadcast(data) {}

  /**
   * Checks if having a connection with a remote peer.
   * @param {string} id - remote peer id
   * @returns {boolean}
   */
  hasConnection(id) {
    return this.connectionTable[id] !== undefined;
  }

  /**
   * Returns the number of connections.
   */
  connectionNum() {
    return this.connections.length;
  }

  // private (protected)

  /**
   * Adds an connection object.
   * @param {string} id - remote peer id
   * @param {object} connection - an object which has remote peer id as .peer property
   * @returns {boolean} if succeeded
   */
  addConnection(id, connection) {
    if (id === this.id || this.connectionTable[id] !== undefined) return false;
    this.connections.push(connection);
    this.connectionTable[id] = connection;
    return true;
  }

  /**
   * Removes an connection object.
   * @param {string} id - remote peer id
   * @returns {boolean} if succeeded
   */
  removeConnection(id) {
    if (id === this.id || this.connectionTable[id] === undefined) return false;
    delete this.connectionTable[id];
    // @TODO: optimize
    let readIndex = 0;
    let writeIndex = 0;
    for (let i = 0, il = this.connections.length; i < il; i++) {
      if (this.connections[readIndex].peer !== id) {
        this.connections[writeIndex] = this.connections[readIndex];
        writeIndex++;
      }
      readIndex++;
    }
    this.connections.length = writeIndex;
    return true;
  }

  // event listeners, refer to .addEventListeners() comment for the arguments.

  invokeOpenListeners(id) {
    this.id = id;
    for (let i = 0, il = this.onOpens.length; i < il; i++) {
      this.onOpens[i](id);
    }
  }

  invokeCloseListeners(id) {
    for (let i = 0, il = this.onCloses.length; i < il; i++) {
      this.onCloses[i](id);
    }
  }

  invokeErrorListeners(error) {
    for (let i = 0, il = this.onErrors.length; i < il; i++) {
      this.onErrors[i](error);
    }
  }

  invokeJoinListeners(hostId) {
    for (let i = 0, il = this.onJoins.length; i < il; i++) {
      this.onJoins[i](hostId);
    }
  }

  invokeRemoteJoinListeners(remoteId) {
    for (let i = 0, il = this.onRemoteJoins.length; i < il; i++) {
      this.onRemoteJoins[i](remoteId);
    }
  }

  invokeRemoteLeaveListeners(remoteId) {
    for (let i = 0, il = this.onRemoteLeaves.length; i < il; i++) {
      this.onRemoteLeaves[i](remoteId);
    }
  }

  invokeConnectListeners(id) {
    for (let i = 0, il = this.onConnects.length; i < il; i++) {
      this.onConnects[i](id);
    }
  }

  invokeDisconnectListeners(id) {
    for (let i = 0, il = this.onDisconnects.length; i < il; i++) {
      this.onDisconnects[i](id);
    }
  }

  invokeReceiveListeners(data) {
    for (let i = 0, il = this.onReceives.length; i < il; i++) {
      this.onReceives[i](data);
    }
  }

  invokeRemoteStreamListeners(stream) {
    for (let i = 0, il = this.onRemoteStreams.length; i < il; i++) {
      this.onRemoteStreams[i](stream);
    }
  }
}

/**
 * WebRTCClient constructor.
 * General WebRTC Client, establishes a connection via Signaling server.
 * @param {THREE.SignalingServer} server
 * @param {object} params - parameters for instantiate (optional)
 */
export default class WebRTCClient extends NetworkClient {
  constructor(server, params={}) {
    super(params);
    this.server = server;
    this.videoStream = null;
    this.audioStream = null;
    this.init();
  }

  /**
   * Initializes signaling server event listener.
   */
  init() {
    // connected with server
    this.server.addEventListener('open', id => {
      this.invokeOpenListeners(id);
    });

    // disconnected from server
    this.server.addEventListener('close', id => {
      this.invokeCloseListeners(id);
    });

    // error occurred with server
    this.server.addEventListener('error', error => {
      this.invokeErrorListeners(error);
    });

    this.server.addEventListener('join', hostId => {
      this.hostId = hostId;
      this.invokeJoinListeners(hostId);
    });

    // aware of a remote peer join the room
    this.server.addEventListener('remote_join', remoteId => {
      if (remoteId === this.id || this.hasConnection(remoteId)) return;

      this.invokeRemoteJoinListeners(remoteId);

      if (this.id !== this.hostId && remoteId !== this.hostId) return;

      // It seems a peer who wants to send stream needs to send a offer
      // @TODO: Confirm if it's true
      const connectFromMe = this.id === this.hostId;
      const peer = new WebRTCPeer(this.id, remoteId, this.server, this.videoStream, this.audioStream);

      // received signal from a remote peer via server
      this.server.addEventListener('receive', signal => {
        peer.handleSignal(signal);
      });

      // connected with a remote peer
      peer.addEventListener('open', id => {
        if (this.addConnection(id, peer)) {
          this.invokeConnectListeners(id, !connectFromMe);
        }
        // TODO: remove server 'receive' listener here.
        // if .addConnection() fails here?
      });

      // disconnected from a remote peer
      peer.addEventListener('close', id => {
        if (this.removeConnection(id)) {
          // TODO: remove server 'receive' listener here.
          this.invokeDisconnectListeners(id);
        }
      });

      // error occurred with a remote peer
      peer.addEventListener('error', error => {
        this.invokeErrorListeners(error);
      });

      // received data from a remote peer
      peer.addEventListener('receive', data => {
        this.invokeReceiveListeners(data);
      });

      // received remote media streaming
      peer.addEventListener('receive_stream', stream => {
        this.invokeRemoteStreamListeners(stream);
      });

      if (connectFromMe) peer.offer();
    });

    this.server.addEventListener('remote_leave', remoteId => {
      this.invokeRemoteLeaveListeners(remoteId);
    });

    // for the compatibility with other NetworkClient classes.
    // if already connected with signaling server, asynchronously invokes open listeners.
    if (this.server.id !== '') {
      requestAnimationFrame(() => {
        this.invokeOpenListeners(this.server.id);
      });
    }
  }

  // public concrete method

  setVideoStream(stream) {
    this.videoStream = stream;
  }

  setAudioStream(stream) {
    this.audioStream = stream;
  }

  join(roomId) {
    this.roomId = roomId;
    this.server.join(roomId);
  }

  send(id, data) {
    const connection = this.connectionTable[id];
    if (connection === undefined) return;
    connection.send(data);
  }

  broadcast(data) {
    for (let i = 0, il = this.connections.length; i < il; i++) {
      this.send(this.connections[i].peer, data);
    }
  }
}

// ice servers for RTCPeerConnection.
const ICE_SERVERS = [
  {urls: 'stun:stun.l.google.com:19302'},
  {urls: 'stun:stun1.l.google.com:19302'},
  {urls: 'stun:stun2.l.google.com:19302'},
  {urls: 'stun:stun3.l.google.com:19302'},
  {urls: 'stun:stun4.l.google.com:19302'}
];

/**
 * WebRTCPeer constructor.
 * WebRTCPeer handles WebRTC connection and data transfer with RTCPeerConnection.
 * Refer to RTCPeerConnection document for the message handling detail.
 * @param {string} id - local peer id
 * @param {string} peer - remote peer id
 * @param {SignalingServer} server
 * @param {MediaStream} videoStream - sends video media stream to remote peer if it's provided (optional)
 * @param {MediaStream} audioStream - sends audio media stream to remote peer if it's provided (optional)
 */
class WebRTCPeer {
  constructor(id, peer, server, videoStream, audioStream) {
    this.id = id;
    this.peer = peer;
    this.server = server;
    this.pc = this.createPeerConnection(videoStream, audioStream);
    this.channel = null;
    this.open = false;

    // event listeners
    this.onOpens = [];
    this.onCloses = [];
    this.onErrors = [];
    this.onReceives = [];
    this.onReceiveStreams = [];
  }

  /**
   * Adds EventListener. Callback function will be invoked when
   * 'open': a connection is established with a remote peer
   * 'close': a connection is disconnected from a remote peer
   * 'error': error occurs
   * 'receive': receives data from a remote peer
   * 'remote_stream': receives a remote media stream
   *
   * Arguments for callback functions are
   * 'open': {string} local peer id
   * 'close': {string} local peer id
   * 'error': {string} error message
   * 'receive': {anything} signal sent from a remote peer
   * 'remote_stream': {MediaStream} remote media stream
   *
   * @param {string} type - event type
   * @param {function} func - callback function
   */
  addEventListener(type, func) {
    switch (type) {
      case 'open':
        this.onOpens.push(func);
        break;
      case 'close':
        this.onCloses.push(func);
        break;
      case 'error':
        this.onErrors.push(func);
        break;
      case 'receive':
        this.onReceives.push(func);
        break;
      case 'receive_stream':
        this.onReceiveStreams.push(func);
        break;
      default:
        console.log('WebRTCPeer.addEventListener: Unknown type ' + type);
        break;
    }
  }

  /**
   * Creates peer connection.
   * @param {MediaStream} videoStream - sends media stream to remote if it's provided (optional)
   * @param {MediaStream} audioStream - sends media stream to remote if it's provided (optional)
   * @returns {RTCPeerConnection}
   */
  createPeerConnection(videoStream, audioStream) {
    const RTCPeerConnection = window.RTCPeerConnection ||
                              window.webkitRTCPeerConnection ||
                              window.mozRTCPeerConnection ||
                              window.msRTCPeerConnection;

    if (RTCPeerConnection === undefined) {
      throw new Error('WebRTCPeer.createPeerConnection: This browser does not seem to support WebRTC.');
    }

    const pc = new RTCPeerConnection({'iceServers': ICE_SERVERS});

    if (videoStream) pc.addStream(videoStream);
    if (audioStream) pc.addStream(audioStream);

    pc.onicecandidate = event => {
      if (event.candidate) {
        const params = {
          id: this.id,
          peer: this.peer,
          type: 'candidate',
          sdpMLineIndex: event.candidate.sdpMLineIndex,
          candidate: event.candidate.candidate
        };
        this.server.send(params);
      }
    };

    pc.onaddstream = event => {
      this.invokeReceiveStreamListeners(event.stream);
    };

    // Note: seems like channel.onclose hander is unreliable on some platforms,
    //       so also try to detect disconnection here.
    pc.oniceconnectionstatechange = () => {
      if(this.open && pc.iceConnectionState === 'disconnected') {
        this.open = false;
        this.invokeCloseListeners(this.peer);
      }
    };

    return pc;
  }

  /**
   * Handles offer request.
   * @param {object} message - message sent from a remote peer
   */
  handleOffer(message) {
    this.pc.ondatachannel = event => {
      this.channel = event.channel;
      this.setupChannelListener();
    };
    this.setRemoteDescription(message);
    this.pc.createAnswer(sdp => {
      this.handleSessionDescription(sdp);
    }, error => {
      console.log('WebRTCPeer.handleOffer: ' + error);
      this.invokeErrorListeners(error);
    });
  }

  /**
   * Handles answer response.
   * @param {object} message - message sent from a remote peer
   */
  handleAnswer(message) {
    this.setRemoteDescription(message);
  }

  /**
   * Handles candidate sent from a remote peer.
   * @param {object} message - message sent from a remote peer
   */
  handleCandidate(message) {
    const RTCIceCandidate = window.RTCIceCandidate ||
                            window.webkitRTCIceCandidate ||
                            window.mozRTCIceCandidate;

    this.pc.addIceCandidate(
      new RTCIceCandidate(message),
      () => {},
      error => {
        console.log('WebRTCPeer.handleCandidate: ' + error);
        this.invokeErrorListeners(error);
      }
    );
  }

  /**
   * Handles SessionDescription.
   * @param {RTCSessionDescription} sdp
   */
  handleSessionDescription(sdp) {
    this.pc.setLocalDescription(sdp, () => {}, error => {
      console.log('WebRTCPeer.handleSessionDescription: ' + error);
      this.invokeErrorListeners(error);
    });

    this.server.send({
      id: this.id,
      peer: this.peer,
      type: sdp.type,
      sdp: sdp.sdp
    });
  }

  /**
   * Sets remote description.
   * @param {object} message - message sent from a remote peer
   */
  setRemoteDescription(message) {
    const RTCSessionDescription = window.RTCSessionDescription ||
                                  window.webkitRTCSessionDescription ||
                                  window.mozRTCSessionDescription ||
                                  window.msRTCSessionDescription;

    this.pc.setRemoteDescription(
      new RTCSessionDescription(message),
      () => {},
      error => {
        console.log('WebRTCPeer.setRemoteDescription: ' + error);
        this.invokeErrorListeners(error);
      }
    );
  }

  /**
   * Sets up channel listeners.
   */
  setupChannelListener() {
    // received data from a remote peer
    this.channel.onmessage = event => {
      this.invokeReceiveListeners(JSON.parse(event.data));
    };

    // connected with a remote peer
    this.channel.onopen = event => {
      this.open = true;
      this.invokeOpenListeners(this.peer);
    };

    // disconnected from a remote peer
    this.channel.onclose = event => {
      if (!this.open) return;
      this.open = false;
      this.invokeCloseListeners(this.peer);
    };

    // error occurred with a remote peer
    this.channel.onerror = error => {
      this.invokeErrorListeners(error);
    };
  }

  // event listeners, refer to .addEventListeners() comment for the arguments.

  invokeOpenListeners(id) {
    for (let i = 0, il = this.onOpens.length; i < il; i++) {
      this.onOpens[i](id);
    }
  }

  invokeCloseListeners(id) {
    for (let i = 0, il = this.onCloses.length; i < il; i++) {
      this.onCloses[i](id);
    }
  }

  invokeErrorListeners(error) {
    for (let i = 0, il = this.onErrors.length; i < il; i++) {
      this.onErrors[i](error);
    }
  }

  invokeReceiveListeners(message) {
    for (let i = 0, il = this.onReceives.length; i < il; i++) {
      this.onReceives[i](message);
    }
  }

  invokeReceiveStreamListeners(stream) {
    for (let i = 0, il = this.onReceiveStreams.length; i < il; i++) {
      this.onReceiveStreams[i](stream);
    }
  }

  // public

  /**
   * Sends connection request (offer) to a remote peer.
   */
  offer() {
    this.channel = this.pc.createDataChannel('mychannel', {reliable: false});
    this.setupChannelListener();
    this.pc.createOffer(sdp => {
      this.handleSessionDescription(sdp);
    }, error => {
      console.log(error);
      this.onError(error);
    });
  }

  /**
   * Sends data to a remote peer.
   * @param {anything} data
   */
  send(data) {
    // TODO: throw error?
    if (this.channel === null || this.channel.readyState !== 'open') return;
    this.channel.send(JSON.stringify(data));
  }

  /**
   * Handles signal sent from a remote peer via server.
   * @param {object} signal - must have .peer as destination peer id and .id as source peer id
   */
  handleSignal(signal) {
    // ignores signal if it isn't for me
    if (this.id !== signal.peer || this.peer !== signal.id) return;

    switch (signal.type) {
      case 'offer':
        this.handleOffer(signal);
        break;
      case 'answer':
        this.handleAnswer(signal);
        break;
      case 'candidate':
        this.handleCandidate(signal);
        break;
      default:
        console.log('WebRTCPeer: Unknown signal type ' + signal.type);
        break;
    }
  }
}
