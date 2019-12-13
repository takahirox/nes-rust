class SignalingServer {
  constructor() {
    this.id = '';  // local peer id, assigned when local peer connects the server
    this.roomId = '';

    // event listeners
    this.onOpens = [];
    this.onJoins = [];
    this.onCloses = [];
    this.onErrors = [];
    this.onRemoteJoins = [];
    this.onRemoteLeaves = [];
    this.onReceives = [];
  }

  /**
   * Adds EventListener. Callback function will be invoked when
   * 'open': a connection is established with a signaling server
   * 'join': joins the room
   * 'close': a connection is disconnected from a signaling server
   * 'error': error occurs
   * 'receive': receives signal from a remote peer via server
   * 'remote_join': aware of a remote peer joins the room
   * 'remote_leave': aware of a remote peer leaves the room
   *
   * Arguments for callback functions are
   * 'open': {string} local peer id
   * 'join': {string} host peer id
   * 'close': {string} local peer id
   * 'error': {string} error message
   * 'receive': {object} signal sent from a remote peer
   * 'remote_join': {string} remote peer id
   * 'remote_leave': {string} remote peer id
   *
   * @param {string} type - event type
   * @param {function} func - callback function
   */
  addEventListener(type, func) {
    switch (type) {
      case 'open':
        this.onOpens.push(func);
        break;
      case 'join':
        this.onJoins.push(func);
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
      case 'remote_join':
        this.onRemoteJoins.push(func);
        break;
      case 'remote_leave':
        this.onRemoteLeaves.push(func);
        break;
      default:
        console.log('SignalingServer.addEventListener: Unknown type ' + type);
        break;
    }
  }

  // invoke event listeners. refer to .addEventListener() comment for arguments.
  invokeOpenListeners(id) {
    for (let i = 0, il = this.onOpens.length; i < il; i++) {
      this.onOpens[i](id);
    }
  }

  invokeJoinListeners(hostUserId) {
    for (let i = 0, il = this.onJoins.length; i < il; i++) {
      this.onJoins[i](hostUserId);
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

  invokeRemoteJoinListeners(id) {
    for (let i = 0, il = this.onRemoteJoins.length; i < il; i++) {
      this.onRemoteJoins[i](id);
    }
  }

  invokeRemoteLeaveListeners(id) {
    for (let i = 0, il = this.onRemoteLeaves.length; i < il; i++) {
      this.onRemoteLeaves[i](id);
    }
  }

  invokeReceiveListeners(signal) {
    for (let i = 0, il = this.onReceives.length; i < il; i++) {
      this.onReceives[i](signal);
    }
  }

  // public abstract method
  /**
   * Joins a room.
   * @param {string} roomId
   */
  join(roomId) {}

  /**
   * Sends signal.
   * TODO: here assumes signal is broadcasted but we should
   *       enable it to send signal to a peer?
   * @param {object} signal
   */
  send(signal) {}
}

/**
 * FirebaseSignalingServer constructor.
 * FirebaseSignalingServer uses Firebase as a signaling server.
 * @param {object} params - parameters for instantiate and Firebase configuration (optional)
 */
export default class FirebaseSignalingServer extends SignalingServer {
  constructor(params={}) {
    super();

    if (window.firebase === undefined) {
      throw new Error('FirebaseSignalingServer: Import firebase from https://www.gstatic.com/firebasejs/x.x.x/firebase.js');
    }

    // Refer to Frebase document for them
    this.apiKey = params.apiKey !== undefined ? params.apiKey : '';
    this.authDomain = params.authDomain !== undefined ? params.authDomain : '';
    this.databaseURL = params.databaseURL !== undefined ? params.databaseURL : '';
    this.authType = params.authType !== undefined ? params.authType : 'anonymous';
    this.init();
    this.auth();
  }

  /**
   * Initializes Firebase.
   */
  init() {
    firebase.initializeApp({
      apiKey: this.apiKey,
      authDomain: this.authDomain,
      databaseURL: this.databaseURL
    });
  }

  /**
   * Authorizes Firebase, depending on authorize type.
   */
  auth() {
    switch (this.authType) {
      case 'none':
        this.authNone();
        break;
      case 'anonymous':
        this.authAnonymous();
        break;
      default:
        console.log('FirebaseSignalingServer.auth: Unknown authType ' + this.authType);
        break;
    }
  }

  /**
   * Doesn't authorize.
   */
  authNone() {
    const generateId = () => {
      let str = '';
      for (let i = 0; i < 16; i++) {
        str += String.fromCharCode(65 + Math.floor(Math.random() * 26));
      }
      return str;
    }
    // makes an unique 16-char id by myself.
    const id = generateId();
    // asynchronously invokes open listeners for the compatibility with other auth types.
    requestAnimationFrame(() => {
      this.id = id;
      this.invokeOpenListeners(id);
    });
  }

  /**
   * Authorizes as anonymous.
   */
  authAnonymous() {
    firebase.auth().signInAnonymously().catch(error => {
      console.error('FirebaseSignalingServer.authAnonymous: ' + error);
      this.invokeErrorListeners(error);
    });

    firebase.auth().onAuthStateChanged(user => {
      if (user === null) {
        // disconnected from server
        this.invokeCloseListeners(this.id);
      } else {
        // authorized
        this.id = user.uid;
        this.invokeOpenListeners(this.id);
      }
    });
  }

  getUserNum(roomId) {
    return new Promise((resolve, reject) => {
      firebase.database().ref(roomId).once('value', ids => {
        resolve(ids.numChildren());
      });
    });
  }

  getHostUser(roomId) {
    return new Promise((resolve, reject) => {
      firebase.database().ref('host/' + roomId).once('value', result => {
        resolve(result.val());
      });
    });
  }

  setHostUser(roomId, id) {
    return new Promise((resolve, reject) => {
      const data = {};
      data[roomId] = id;
      firebase.database().ref('host').set(data, () => {
        firebase.database().ref('host/' + roomId).onDisconnect().remove();
        resolve(id)
      });
    });
  }

  // public concrete method

  join(roomId) {
    this.roomId = roomId;
    // @TODO: take care concurrent access
    this.getUserNum(roomId).then(userNum => {
      const isHost = userNum === 0;
      return isHost ? this.setHostUser(roomId, this.id) : this.getHostUser(roomId);
    }).then(hostUserId => {
      this.invokeJoinListeners(hostUserId);
      const ref = firebase.database().ref(this.roomId + '/' + this.id);
      ref.set({signal: ''});
      ref.onDisconnect().remove();
      const doneTable = {};  // remote peer id -> true or undefined, indicates if already done.
      firebase.database().ref(this.roomId).on('child_added', data => {
        const id = data.key;
        if (id === this.id || doneTable[id] === true) {
          return;
        }
        doneTable[id] = true;
        // received signal
        firebase.database().ref(this.roomId + '/' + id + '/signal').on('value', data => {
          if (data.val() === null || data.val() === '') {
            return;
          }
          this.invokeReceiveListeners(data.val());
        });
        this.invokeRemoteJoinListeners(id);
      });
      firebase.database().ref(roomId).on('child_removed', data => {
        delete doneTable[data.key];
        this.invokeRemoteLeaveListeners(data.key);
      });
    });
  }

  // TODO: we should enable .send() to send signal to a peer, not only broadcast?
  send(data) {
    firebase.database().ref(this.roomId + '/' + this.id + '/signal').set(data);
  }
}
