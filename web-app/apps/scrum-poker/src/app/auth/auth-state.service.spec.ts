import { TestBed } from "@angular/core/testing";
import * as sinon from "sinon";

import { AuthStateService, NotLoggedInError } from "./auth-state.service";
import { User } from "../../types";
import { AuthEventService, EventType, LoggedOutReason } from "./auth-event.service";
import { AuthStorageService } from "./auth-storage.service";
import { nextEventLoop } from "../utils";

describe("AuthStateService", () => {
    let sandbox: sinon.SinonSandbox;
    let fakeAuthStorage: {
        getItem: sinon.SinonStub;
        setItem: sinon.SinonStub;
        removeItem: sinon.SinonStub;
        clear: sinon.SinonStub;
    };

    const user: Readonly<User> = {
        userId: "0123-4567-8901-2345",
        name: "Calvin"
    };
    const prevAccessToken: Readonly<string> = "prev-access-token";
    const accessToken: Readonly<string> = "access-token";

    beforeEach(() => {
        fakeAuthStorage = {
            getItem: sinon
                .stub()
                .withArgs("access_token")
                .resolves(prevAccessToken),
            setItem: sinon.stub().resolves(),
            removeItem: sinon.stub().resolves(),
            clear: sinon.stub().resolves()
        };
        TestBed.configureTestingModule({
            providers: [
                {
                    provide: AuthStorageService,
                    useValue: fakeAuthStorage
                }
            ]
        });

        sandbox = sinon.createSandbox();
        sandbox.stub(console, "error");
    });

    afterEach(() => {
        sandbox.restore();
    });

    it("should emit AppStateInitialized event after retrieving access token from AuthStorage", async done => {
        fakeAuthStorage.getItem.withArgs("access_token").resolves(accessToken);

        TestBed.configureTestingModule({
            providers: [
                {
                    provide: AuthStorageService,
                    useValue: fakeAuthStorage
                }
            ]
        });

        const authState: AuthStateService = TestBed.get(AuthStateService);

        await nextEventLoop();

        const authEvent: AuthEventService = TestBed.get(AuthEventService);
        authEvent.$event.subscribe(event => {
            expect(authState.accessToken).toEqual(accessToken);
            expect(event).toEqual({
                type: EventType.AuthStateInitialized
            });

            done();
        });
    });

    describe("When LoggedInEvent is emitted", () => {
        it("should login with user and accessToken from event value", async () => {
            const authState: AuthStateService = TestBed.get(AuthStateService);

            await newLoggedInAuthState(user, accessToken);

            expect(authState.isLoggedIn).toBeTruthy();
            expect(authState.user).toEqual(user);
            expect(authState.accessToken).toEqual(accessToken);
        });

        it("should persist accessToken to AuthStorage", async () => {
            await newLoggedInAuthState(user, accessToken);

            expect(fakeAuthStorage.setItem.callCount).toEqual(1);
            expect(fakeAuthStorage.setItem.firstCall.args).toEqual([
                "access_token",
                accessToken
            ]);
        });

        it("should update user and accessToken regardless of persistence to AuthStorage", async () => {
            fakeAuthStorage.setItem.resetBehavior();
            fakeAuthStorage.setItem.rejects(new Error("Storage error"));
            const authState: AuthStateService = TestBed.get(AuthStateService);

            await newLoggedInAuthState(user, accessToken);

            expect(fakeAuthStorage.setItem.callCount).toEqual(1);
            expect(fakeAuthStorage.setItem.firstCall.args).toEqual([
                "access_token",
                accessToken
            ]);

            expect(authState.isLoggedIn).toBeTruthy();
            expect(authState.user).toEqual(user);
            expect(authState.accessToken).toEqual(accessToken);
        });
    });

    describe("when LoggedOutEvent is emitted", () => {
        it("should logout user when LoggedOutEvent is emitted", async () => {
            const authState: AuthStateService = TestBed.get(AuthStateService);
            const authEvent: AuthEventService = TestBed.get(AuthEventService);

            await newLoggedInAuthState(user, accessToken);

            expect(authState.isLoggedIn).toBeTruthy();

            authEvent.loggedOut(LoggedOutReason.SessionExpired);

            await nextEventLoop();

            expect(authState.isLoggedIn).toBeFalsy();
            expect(() => authState.user).toThrow(NotLoggedInError);
        });

        it("should clear AuthStorage", async () => {
            const authEvent: AuthEventService = TestBed.get(AuthEventService);

            await newLoggedInAuthState(user, accessToken);

            authEvent.loggedOut(LoggedOutReason.SessionExpired);

            await nextEventLoop();

            expect(fakeAuthStorage.clear.callCount).toEqual(1);
        });

        it("should reset user and accessToken regardless of clearance of AuthStorage", async () => {
            fakeAuthStorage.clear.resetBehavior();
            fakeAuthStorage.clear.rejects(new Error("Storage error"));
            const authEvent: AuthEventService = TestBed.get(AuthEventService);
            const authState: AuthStateService = TestBed.get(AuthStateService);

            await newLoggedInAuthState(user, accessToken);

            authEvent.loggedOut(LoggedOutReason.SessionExpired);

            await nextEventLoop();

            expect(fakeAuthStorage.clear.callCount).toEqual(1);

            expect(authState.isLoggedIn).toBeFalsy();
            expect(() => authState.user).toThrow(NotLoggedInError);
        });
    });

    describe("when accessTokenRefreshed event is emitted", () => {
        it("should refuse to refresh access token when user is not logged in before", async () => {
            const authEvent: AuthEventService = TestBed.get(AuthEventService);
            const authState = await newAuthState();

            authEvent.accessTokenRefreshed(accessToken);

            expect(authState.isLoggedIn).toBeFalsy();
            expect(() => authState.user).toThrow(NotLoggedInError);
            expect(authState.accessToken).toEqual(prevAccessToken);
        });

        it("should refresh access token", async () => {
            const authEvent: AuthEventService = TestBed.get(AuthEventService);

            const authState = await newLoggedInAuthState(user, accessToken);

            authEvent.accessTokenRefreshed(accessToken);

            expect(authState.accessToken).toEqual(accessToken);
        });

        it("should persist access token to AuthStorage", async () => {
            const authEvent: AuthEventService = TestBed.get(AuthEventService);

            await newLoggedInAuthState(user, accessToken);

            authEvent.accessTokenRefreshed(accessToken);

            await nextEventLoop();

            expect(fakeAuthStorage.setItem.callCount).toEqual(1);
            expect(fakeAuthStorage.setItem.firstCall.args).toEqual([
                "access_token",
                accessToken
            ]);
        });

        it("should refresh access token regardless of persistence of AuthStorage", async () => {
            fakeAuthStorage.clear.resetBehavior();
            fakeAuthStorage.clear.rejects(new Error("Storage error"));
            const authEvent: AuthEventService = TestBed.get(AuthEventService);
            const authState: AuthStateService = await newLoggedInAuthState(
                user,
                accessToken
            );

            authEvent.accessTokenRefreshed(accessToken);

            await nextEventLoop();

            expect(fakeAuthStorage.setItem.callCount).toEqual(1);
            expect(authState.accessToken).toEqual(accessToken);
        });
    });

    const newLoggedInAuthState = async (
        eventUser: User,
        eventAccessToken: string
    ): Promise<AuthStateService> => {
        const authEvent: AuthEventService = TestBed.get(AuthEventService);
        const service = await newAuthState();

        authEvent.loggedIn(eventUser, eventAccessToken);

        await nextEventLoop();

        return service;
    };

    const newAuthState = async (): Promise<AuthStateService> => {
        const service: AuthStateService = TestBed.get(AuthStateService);
        await nextEventLoop();

        return service;
    };
});
