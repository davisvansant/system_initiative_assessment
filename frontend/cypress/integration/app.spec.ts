// https://docs.cypress.io/api/introduction/api.html

describe('system initiative assessment tests', () => {
  it('visits the app root url', () => {
    cy.visit('/')
    cy.title().should('eq', 'welcome to the system initiative')
    cy.get('#system_init').should('have.class', 'system_init')
    cy.get('#title').should('have.class', 'title')
    cy.get('#close_connection').should('have.class', 'close_connection')
    cy.get('#new_message_input').should('have.class', 'new_message_input')
    cy.get('#send_new_message').should('have.class','send_new_message')
    cy.get('#incoming_messages').should('have.class', 'incoming_messages')
  })
})
