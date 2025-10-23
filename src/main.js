const { listen } = window.__TAURI__.event;
const invoke = window.__TAURI__.core.invoke;

let tokensEl;
let claimEl;
let proxiesEl;
let logsEl;
let nameEl;
let fastEl;
let privEl;
let startEl;
let stopEl;

window.addEventListener("DOMContentLoaded", () => {
  tokensEl = document.querySelector("#tokens");
  claimEl = document.querySelector("#claim_token");
  proxiesEl = document.querySelector("#proxies");
  logsEl = document.querySelector("#logs");
  nameEl = document.querySelector("#name");
  stopEl = document.querySelector("#stop");
  stopEl.addEventListener("click", stop);
  startEl = document.querySelector("#start");
  startEl.addEventListener("click", start);
});

listen("log", (event) => {
  var move = false;
  if (logsEl.scrollTop >= logsEl.scrollHeight - logsEl.clientHeight - 1) {
    move = true;
  }
  logsEl.innerHTML += `${event.payload}\n`;
  if (move) {
    logsEl.scrollTop = logsEl.scrollHeight;
  }
});

listen("stop", (event) => {
  stopEl.style.setProperty("display", "none");
  startEl.style.setProperty("display", "block");
});

listen("alert", (event) => {
  alert(event.payload);
});

function start() {
  var data = {
    claim: claimEl.value,
    accounts: tokensEl.value.split("\n").filter((line) => line.trim() !== ""),
    proxies: proxiesEl.value.split("\n").filter((line) => line.trim() !== ""),
    name: nameEl.value,
  };
  invoke("start", data).then((res) => {
    if (res) {
      startEl.style.setProperty("display", "none");
      stopEl.style.setProperty("display", "block");
    }
  });
}

function stop() {
  invoke("stop");
}
