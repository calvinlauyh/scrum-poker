import { getGreeting } from '../support/app.po';

describe('scrum-poker', () => {
  beforeEach(() => cy.visit('/'));

  it('should display welcome message', () => {
    getGreeting().contains('Welcome to scrum-poker!');
  });
});
