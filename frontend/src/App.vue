<template>
  <div>
    <input type="text" id="new_message" class="new_message" autofocus @keyup.enter="sendMessage()">
    <br>
    <button v-on:click="sendMessage()">new message</button>
    <br>
    <li v-for="message in messages" :key="message">
      {{ message }}
    </li>
    <button v-on:click="closeConnection()">close connection</button>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { webSocket } from 'rxjs/webSocket'

const url = 'ws://localhost:5555/messages'
const connection = webSocket(url)
const messages:string[] = []

export default defineComponent({
  name: 'app',
  data () {
    return {
      connection: webSocket.prototype,
      messages
    }
  },
  created () {
    this.connection = connection

    connection.subscribe({
      next: message => {
        const newMessage = message as JSON

        console.log('message received: ' + JSON.stringify(newMessage))
        console.log('message received: ' + newMessage)

        this.receiveMessage(message as string)
      },
      error: error => console.log(error),
      complete: () => console.log('connection closed')
    })
  },
  methods: {
    closeConnection () {
      this.connection.error({ code: 1000, reason: 'goodbye!' })
    },
    receiveMessage (incomingMessage: string) {
      this.messages.push(incomingMessage)

      return this.messages
    },
    sendMessage () {
      const newMessage = document.getElementById('new_message') as HTMLInputElement

      if (newMessage != null) {
        this.connection.next(newMessage.value)

        const clear = newMessage.value = ''

        return clear
      }
    }
  }
})
</script>
