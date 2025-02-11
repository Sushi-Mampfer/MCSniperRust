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
  fastEl = document.querySelector("#fast");
  privEl = document.querySelector("#private");
  stopEl = document.querySelector("#stop");
  stopEl.addEventListener("click", stop);
  startEl = document.querySelector("#start");
  startEl.addEventListener("click", start);
  setInterval(function () {
    var height = window.getComputedStyle(
      document.querySelector(".switch_cb"),
      "::after",
    ).height;
    var width = document.querySelector(".switch_cb").clientWidth;
    document.documentElement.style.setProperty("--switch-height", height);
    document.documentElement.style.setProperty(
      "--switch-move",
      parseFloat(width, 10) - parseFloat(height, 10) + "px",
    );
  }, 100);
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
    accounts: tokensEl.value.split("\n").filter((line) => line.trim() !== ""),
    claim: claimEl.value,
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
