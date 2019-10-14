import { Injectable } from "@angular/core";
import { User, AUTH_ERROR_CODE, LOGIN_ERROR_CODE } from "../../types";
import {
    AuthEventService,
    Event,
    EventType,
    LoggedInEvent
} from "./auth-event.service";
import { AuthStorageService } from "./auth-storage.service";

@Injectable({
    providedIn: "root"
})
export class AuthStateService {
    public $event: Event;

    private _user: User;
    private _accessToken: string;

    private readonly AUTH_STORAGE_LAST_PROVIDER_ID_KEY = "last_provider_id";
    private readonly AUTH_STORAGE_ACCESS_TOKEN_KEY = "access_token";

    constructor(
        private authEvent: AuthEventService,
        private authStorage: AuthStorageService
    ) {
        this.listenToEvent();

        this.initialize();
    }

    private listenToEvent(): void {
        this.authEvent.$event.subscribe(event => {
            this.handleEvent(event);
        });
    }

    private handleEvent(event: Event) {
        switch (event.type) {
            case EventType.LoggedIn:
                this.handleLoggedInEvent(event);
                break;
            case EventType.LoggedOut:
                this.handleLoggedOutEvent(event);
                break;
        }
    }

    private handleLoggedInEvent(event: LoggedInEvent): void {
        this._user = event.user;
        this._accessToken = event.accessToken;

        // TODO: Error handling
        this.authStorage
            .setItem(this.AUTH_STORAGE_ACCESS_TOKEN_KEY, event.accessToken)
            .catch(console.error);
    }

    private handleLoggedOutEvent(event): void {
        this._user = null;
        this._accessToken = "";

        // TODO: Error handling
        this.authStorage.clear().catch(console.error);
    }

    private initialize(): void {
        // TODO: Error handling
        this.authStorage
            .getItem(this.AUTH_STORAGE_ACCESS_TOKEN_KEY)
            .then(accessToken => {
                this._accessToken = accessToken;
                this.authEvent.authStateInitialized();
            })
            .catch(console.error);
    }

    public get isLoggedIn(): boolean {
        return !!this._user;
    }

    public get user(): User {
        if (!this.isLoggedIn) {
            throw new NotLoggedInError();
        }

        return this._user;
    }

    public get accessToken(): string {
        return this._accessToken;
    }
}

export class NotLoggedInError extends Error {
    constructor() {
        super("User not logged in");
    }
}
