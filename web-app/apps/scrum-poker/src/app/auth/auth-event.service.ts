import { Injectable } from "@angular/core";
import { BehaviorSubject } from "rxjs";
import { AUTH_ERROR_CODE, User, LOGIN_ERROR_CODE, Omit } from "../../types";

@Injectable({
    providedIn: "root"
})
export class AuthEventService {
    private _$event: BehaviorSubject<Event>;

    constructor() {
        this._$event = new BehaviorSubject({
            type: EventType.Initial
        });
    }

    public get $event(): Omit<
        Omit<BehaviorSubject<Event>, "next">,
        "complete"
    > {
        return this._$event;
    }

    public authStateInitialized(): void {
        this._$event.next({
            type: EventType.AuthStateInitialized
        });
    }

    public authSucceeded(providerId: string, state: any): void {
        this._$event.next({
            type: EventType.AuthSucceeded,
            providerId,
            state
        });
    }

    public authFailed(providerId: string, errorCode: AUTH_ERROR_CODE): void {
        this._$event.next({
            type: EventType.AuthFailed,
            providerId,
            errorCode
        });
    }

    public loggedIn(user: User, accessToken: string): void {
        this._$event.next({
            type: EventType.LoggedIn,
            user,
            accessToken
        });
    }

    public logInFailed(errorCode: LOGIN_ERROR_CODE): void {
        this._$event.next({
            type: EventType.LogInFailed,
            errorCode
        });
    }

    public loggedOut(reason: LoggedOutReason): void {
        this._$event.next({
            type: EventType.LoggedOut,
            reason
        });
    }

    public accessTokenRefreshed(accessToken: string): void {
        this._$event.next({
            type: EventType.AccessTokenRefreshed,
            accessToken
        });
    }
}

export type Event =
    | InitialEvent
    | AuthStateInitializedEvent
    | AuthSucceededEvent
    | AuthFailedEvent
    | LoggedInEvent
    | LogInFailedEvent
    | LoggedOutEvent
    | AccessTokenRefreshedEvent;

export interface BaseEvent {
    type: EventType;
}

export interface InitialEvent extends BaseEvent {
    type: EventType.Initial;
}

export interface AuthStateInitializedEvent extends BaseEvent {
    type: EventType.AuthStateInitialized;
}

export interface AuthSucceededEvent extends BaseEvent {
    type: EventType.AuthSucceeded;
    providerId: string;
    state: any;
}

export interface AuthFailedEvent extends BaseEvent {
    type: EventType.AuthFailed;
    providerId: string;
    errorCode: AUTH_ERROR_CODE;
}

export interface LoggedInEvent extends BaseEvent {
    type: EventType.LoggedIn;
    user: User;
    accessToken: string;
}

export interface LogInFailedEvent extends BaseEvent {
    type: EventType.LogInFailed;
    errorCode: LOGIN_ERROR_CODE;
}

export interface LoggedOutEvent extends BaseEvent {
    type: EventType.LoggedOut;
    reason: LoggedOutReason;
}

export interface AccessTokenRefreshedEvent {
    type: EventType.AccessTokenRefreshed;
    accessToken: string;
}

export enum EventType {
    Initial,
    AuthStateInitialized,
    AuthSucceeded,
    AuthFailed,
    LoggedIn,
    LogInFailed,
    LoggedOut,
    AccessTokenRefreshed
}

export enum LoggedOutReason {
    UserLogout,
    SessionExpired
}
