import { Component, OnInit } from "@angular/core";

import { AuthService } from "./auth/auth.service";
import { GoogleAuthProvider } from "./auth/provider/google-auth-provider";
import { AuthEventService, Event as AuthEvent, EventType } from "./auth/auth-event.service";

@Component({
  selector: "web-app-root",
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.scss"]
})
export class AppComponent implements OnInit {
  title = "Scrum Poker";
  action: string;

  constructor(
    private authService: AuthService,
    private authEvent: AuthEventService
  ) {}

  ngOnInit() {
    this.listenToAuthEvent();
  }

  private listenToAuthEvent() {
    this.authEvent.$event.subscribe(this.handleAuthEvent);
  }

  private handleAuthEvent(event: AuthEvent) {
    switch (event.type) {
      case EventType.AuthSucceeded:
        this.action = "Auth succeeded";
        break;
      case EventType.AuthFailed:
        this.action = "Auth failed";
        break;
      case EventType.LoggedIn:
        this.action = "Loggedin";
        break;
    }
  }

  public login() {
    this.authService.tryAuthAndLogin(GoogleAuthProvider.PROVIDER_ID);
  }
}
