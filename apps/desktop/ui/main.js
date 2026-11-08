const { invoke } = window.__TAURI__.core;

const chat = document.getElementById("chat");
const form = document.getElementById("composer");
const input = document.getElementById("prompt");

function appendMessage(role, text) {
  const el = document.createElement("div");
  el.className = `chat__message chat__message--${role}`;
  el.textContent = text;
  chat.appendChild(el);
  chat.scrollTop = chat.scrollHeight;
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  const message = input.value.trim();
  if (!message) return;

  appendMessage("user", message);
  input.value = "";

  try {
    const reply = await invoke("send_message", { message });
    appendMessage("assistant", reply);
  } catch (err) {
    appendMessage("assistant", `Error: ${err}`);
  }
});
