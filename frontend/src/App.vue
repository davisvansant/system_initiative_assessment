<template>
  <div id="system_init" class="system_init">
    <div id="title" class="title">
      <span class="welcome">welcome to the system initative</span>
      <button
        id="close_connection"
        class="close_connection"
        @click="closeConnection()"
      >
        close connection
      </button>
    </div>
    <hr class="hr" />
    <div class="new_messages">
      <span class="new_message">new message</span>
      <input
        type="text"
        class="new_message_input"
        id="new_message_input"
        @keyup.enter="sendMessage()"
      />
      <button
        id="send_new_message"
        class="send_new_message"
        @click="sendMessage()"
      >
        send
      </button>
    </div>
    <div id="incoming_messages" class="incoming_messages">
      <p
        id="message"
        class="message"
        v-for="message in messages"
        :key="message"
      >
        {{ message }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineComponent } from "vue";
import { webSocket } from "rxjs/webSocket";
</script>

<script lang="ts">
export default defineComponent({
  name: "app",
  data() {
    return {
      connection: webSocket.prototype,
      messages: [] as string[],
    };
  },
  created() {
    const url = "ws://localhost:5555/messages";
    const connection = webSocket(url);
    const messages: string[] = [];

    this.messages = messages;
    this.connection = connection;

    connection.subscribe({
      next: (message) => {
        this.receiveMessage(message as string);
      },
      error: (error) => console.log(error),
      complete: () => console.log("connection closed"),
    });
  },
  methods: {
    closeConnection() {
      this.connection.error({ code: 1000, reason: "goodbye!" });
    },
    receiveMessage(incomingMessage: string) {
      this.messages.push(incomingMessage);

      return this.messages;
    },
    sendMessage() {
      const newMessage = document.getElementById(
        "new_message_input"
      ) as HTMLInputElement;

      console.log("new message ->", newMessage.value);

      if (newMessage != null) {
        this.connection.next(newMessage.value);

        const clear = (newMessage.value = "");

        return clear;
      }
    },
  },
});
</script>

<style>
body {
  background-color: black;
}

.system_init {
  display: flex;
  flex-direction: column;
  padding: 25px;
  height: 75vh;
  margin: 5vh;
  border: 1px solid aquamarine;
  border-radius: 25px;
}

.title {
  display: flex;
  flex-direction: row;
  align-items: none;
  justify-content: space-between;
}

.welcome {
  color: snow;
  font-family: "Courier New", Courier, monospace;
  font-size: 16px;
  letter-spacing: 5px;
}

.close_connection {
  color: snow;
  background-color: transparent;
  font-family: Arial, Helvetica, sans-serif;
  letter-spacing: 1px;
  border: 2px solid transparent;
  border-radius: 2px;
  border-left: 2px solid aquamarine;
  border-right: 2px solid aquamarine;
  transition-duration: 1s;
}

.close_connection:hover {
  color: black;
  background: aquamarine;
  border: 2px solid aquamarine;
}

.hr {
  width: 95%;
  border: 1px solid dimgray;
  margin: 15px;
}

.new_messages {
  display: flex;
  flex-direction: row;
  align-items: center;
  align-self: center;
  width: 75%;
}

.new_message {
  color: aquamarine;
  font-family: Arial, Helvetica, sans-serif;
  font-size: 12px;
  letter-spacing: 2px;
  padding-right: 20px;
}

.new_message_input {
  width: 65%;
  height: 15px;
  background: snow;
  border: transparent;
}

.send_new_message {
  color: black;
  background-color: dimgray;
  border: 1px solid transparent;
  border-radius: 10px;
  width: 75px;
  margin: 15px;
  padding: 5px;
  transition-duration: 0.5s;
}

.send_new_message:hover {
  background: aquamarine;
}

.incoming_messages {
  display: flex;
  align-self: center;
  align-items: center;
  overflow: auto;
  flex-direction: column;
  width: 85%;
  height: 85%;
}

.message {
  font-family: "Courier New", Courier, monospace;
  font-size: 12px;
  color: snow;
  box-shadow: 4px 4px 8px aquamarine;
  border-radius: 5px;
  width: 75%;
  height: 20px;
  text-align: center;
  padding: 5px;
}
</style>
