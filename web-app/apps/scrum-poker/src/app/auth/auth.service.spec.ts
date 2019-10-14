import { TestBed } from "@angular/core/testing";
import * as sinon from "sinon";
import { HttpClientTestingModule } from "@angular/common/http/testing";
import { Injectable } from "@angular/core";
import { defer } from "rxjs";

import {
  AuthService,
  AuthServiceConfig,
  ProviderNotFoundError
} from "./auth.service";
import { Class, User, ERROR_CODE, LOGIN_ERROR_CODE } from "../../types";
import { AuthProvider, AuthResult, AuthStatus } from "./provider/auth-provider";
import { ServerApiService, ServerApiError } from "../server/server-api.service";
import { AuthStorageService } from "./auth-storage.service";
import { nextEventLoop } from "../utils";
import {
  AuthEventService,
  EventType,
  LoggedOutReason
} from "./auth-event.service";
import { AuthStateService } from "./auth-state.service";

class DummyInjectable {}
@Injectable()
class DummyAuthProvider extends AuthProvider {
  public static PROVIDER_ID = "Dummy";
  constructor() {
    super();
  }
  public tryAuth(): Promise<AuthResult> {
    return Promise.resolve({
      status: AuthStatus.Succeeded,
      state: {}
    });
  }
  public verify(): Promise<AuthResult> {
    return Promise.resolve({
      status: AuthStatus.Succeeded,
      state: {}
    });
  }
}

describe("AuthService", () => {
  let sandbox: sinon.SinonSandbox;
  let fakeAuthStorage: {
    getItem: sinon.SinonStub;
    setItem: sinon.SinonStub;
    removeItem: sinon.SinonStub;
    clear: sinon.SinonStub;
  };
  beforeEach(() => {
    fakeAuthStorage = {
      getItem: sinon.stub().rejects(new Error("getItem not implemented")),
      setItem: sinon.stub().resolves(),
      removeItem: sinon.stub().resolves(),
      clear: sinon.stub().resolves()
    };
    TestBed.configureTestingModule({
      imports: [HttpClientTestingModule],
      providers: [
        {
          provide: AuthServiceConfig,
          useValue: new AuthServiceConfig([])
        },
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

  describe("tryAuthAngLogin()", () => {
    describe("When provider does not exist", () => {
      it("should throw ProviderNotFoundError", () => {
        const service: AuthService = TestBed.get(AuthService);

        const providerId = DummyAuthProvider.PROVIDER_ID;
        return expect(service.tryAuthAndLogin(providerId)).rejects.toThrowError(
          ProviderNotFoundError
        );
      });

      it("should emit AuthFailed event", done => {
        const authService: AuthService = TestBed.get(AuthService);
        const authEvent: AuthEventService = TestBed.get(AuthEventService);

        const providerId = DummyAuthProvider.PROVIDER_ID;
        authService.tryAuthAndLogin(providerId).catch(() => {
          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthFailed,
              providerId,
              errorCode: ERROR_CODE.AUTH_PROVIDER_NOT_FOUND
            });
          });

          done();
        });
      });
    });

    describe("When the provider is not injectable", () => {
      beforeEach(() => {
        TestBed.configureTestingModule({
          providers: [
            {
              provide: AuthServiceConfig,
              useValue: new AuthServiceConfig([DummyAuthProvider])
            }
          ]
        });
      });

      it("should throw Injector Error", () => {
        const service: AuthService = TestBed.get(AuthService);

        return expect(
          service.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID)
        ).rejects.toThrowError(
          `No provider for ${DummyAuthProvider.PROVIDER_ID}`
        );
      });

      it("should emit AuthFailed event", done => {
        const authService: AuthService = TestBed.get(AuthService);
        const authEvent: AuthEventService = TestBed.get(AuthEventService);

        const providerId = DummyAuthProvider.PROVIDER_ID;
        authService.tryAuthAndLogin(providerId).catch(() => {
          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthFailed,
              providerId,
              errorCode: ERROR_CODE.AUTH_PROVIDER_NOT_INJECTABLE
            });
          });

          done();
        });
      });
    });

    it("should inject the Provider", async () => {
      let injected = false;
      let injectedDummyInjectable = null;
      @Injectable()
      class FooProvider extends DummyAuthProvider {
        constructor(public dummyInjectable: DummyInjectable) {
          super();
          injected = true;
          injectedDummyInjectable = dummyInjectable;
        }
      }

      configureDummyAuthProviderInTestBed(FooProvider);
      const service: AuthService = TestBed.get(AuthService);

      await service.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);

      expect(injected).toBeTruthy();
      expect(injectedDummyInjectable).toBeInstanceOf(DummyInjectable);
    });

    it("should invoke tryAuth of injected provider", async () => {
      let callCount = 0;
      @Injectable()
      class FooProvider extends DummyAuthProvider {
        public async tryAuth(): Promise<AuthResult> {
          callCount += 1;
          return { status: AuthStatus.Succeeded, state: {} };
        }
      }
      configureDummyAuthProviderInTestBed(FooProvider);
      const service: AuthService = TestBed.get(AuthService);

      await service.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);

      expect(callCount).toEqual(1);
    });

    describe("AuthStorage last auth provider", () => {
      let tryAuthStub: sinon.SinonStub;
      @Injectable()
      class AuthFailedProvider extends DummyAuthProvider {
        public async tryAuth(): Promise<AuthResult> {
          return (tryAuthStub as (() => Promise<AuthResult>))();
        }
      }
      beforeEach(() => {
        tryAuthStub = sinon.stub();
        configureDummyAuthProviderInTestBed(AuthFailedProvider);
      });

      it("should update AuthStorage last_provider_id before tryAuthAndLogin resolve and remove last_provider_id after resolves to success result", async () => {
        let resolveTryAuth: any;
        tryAuthStub.resolves(
          new Promise(resolve => {
            resolveTryAuth = resolve;
          })
        );

        const service: AuthService = TestBed.get(AuthService);

        service.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);

        await nextEventLoop();

        expect(fakeAuthStorage.setItem.callCount).toEqual(1);
        expect(fakeAuthStorage.setItem.firstCall.args).toEqual([
          "last_provider_id",
          DummyAuthProvider.PROVIDER_ID
        ]);

        resolveTryAuth({
          status: AuthStatus.Succeeded,
          state: {
            authCode: "1234567890"
          }
        } as AuthResult);

        await nextEventLoop();

        expect(fakeAuthStorage.removeItem.callCount).toEqual(1);
        expect(fakeAuthStorage.removeItem.firstCall.args).toEqual([
          "last_provider_id"
        ]);
      });

      it("should update AuthStorage last_provider_id before tryAuthAndLogin resolve and remove last_provider_id after resolves to fail result", async () => {
        let resolveTryAuth: (value?: any) => void;
        tryAuthStub.resolves(
          new Promise(resolve => {
            resolveTryAuth = resolve;
          })
        );

        const service: AuthService = TestBed.get(AuthService);

        service.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);

        await nextEventLoop();

        expect(fakeAuthStorage.setItem.callCount).toEqual(1);
        expect(fakeAuthStorage.setItem.firstCall.args).toEqual([
          "last_provider_id",
          DummyAuthProvider.PROVIDER_ID
        ]);

        resolveTryAuth({
          status: AuthStatus.Failed,
          errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
        } as AuthResult);

        await nextEventLoop();

        expect(fakeAuthStorage.removeItem.callCount).toEqual(1);
        expect(fakeAuthStorage.removeItem.firstCall.args).toEqual([
          "last_provider_id"
        ]);
      });
    });

    describe("When tryAutAndLogin resolves to postponed auth result", () => {
      @Injectable()
      class AuthPostponedProvider extends DummyAuthProvider {
        public async tryAuth(): Promise<AuthResult> {
          return {
            status: AuthStatus.Postponed
          };
        }
      }

      it("should not emit any event", async done => {
        configureDummyAuthProviderInTestBed(AuthPostponedProvider);
        const authService: AuthService = TestBed.get(AuthService);
        const authEvent: AuthEventService = TestBed.get(AuthEventService);

        await authService.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);
        authEvent.$event.subscribe(event => {
          expect(event).toEqual({
            type: EventType.Initial
          });
          done();
        });
      });
    });

    describe("When tryAuthAndLogin resolves to failed auth result", () => {
      @Injectable()
      class AuthFailedProvider extends DummyAuthProvider {
        public async tryAuth(): Promise<AuthResult> {
          return {
            status: AuthStatus.Failed,
            errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
          };
        }
      }

      it("should emit AuthFailed event when tryAuthAndLogin resolves to failed auth result", async done => {
        configureDummyAuthProviderInTestBed(AuthFailedProvider);
        const authService: AuthService = TestBed.get(AuthService);
        const authEvent: AuthEventService = TestBed.get(AuthEventService);

        await authService.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);
        authEvent.$event.subscribe(event => {
          expect(event).toEqual({
            type: EventType.AuthFailed,
            providerId: AuthFailedProvider.PROVIDER_ID,
            errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
          });
          done();
        });
      });
    });

    describe("When tryAuthAndLogin resolves to success auth result", () => {
      const authState = {
        authCode: "1234567890"
      };
      @Injectable()
      class AuthFailedProvider extends DummyAuthProvider {
        public async tryAuth(): Promise<AuthResult> {
          return { status: AuthStatus.Succeeded, state: authState };
        }
      }

      it("should emit AuthSucceeded event", done => {
        configureDummyAuthProviderInTestBed(AuthFailedProvider);
        const authService: AuthService = TestBed.get(AuthService);
        const authEvent: AuthEventService = TestBed.get(AuthEventService);

        authService.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID).then(() => {
          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthSucceeded,
              providerId: AuthFailedProvider.PROVIDER_ID,
              state: authState
            });
            done();
          });
        });
      });
    });

    describe("When auth succeeded", () => {
      const authState: Readonly<any> = {
        authCode: "1234567890"
      };
      let authService: AuthService;
      let serverApiServiceLoginStub: sinon.SinonStub;
      beforeEach(() => {
        @Injectable()
        class AuthSucceededProvider extends DummyAuthProvider {
          public async tryAuth(): Promise<AuthResult> {
            return { status: AuthStatus.Succeeded, state: authState };
          }
        }
        configureDummyAuthProviderInTestBed(AuthSucceededProvider);

        serverApiServiceLoginStub = sinon.stub();
        TestBed.configureTestingModule({
          providers: [
            {
              provide: ServerApiService,
              useValue: {
                login: serverApiServiceLoginStub
              }
            }
          ]
        });

        authService = TestBed.get(AuthService);
      });

      it("should login to the server", async () => {
        const user: Readonly<User> = {
          userId: "0123-4567-8901-2345",
          name: "Calvin"
        };
        serverApiServiceLoginStub.returns(
          defer(() => Promise.resolve({ user }))
        );

        await authService.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);

        expect(serverApiServiceLoginStub.callCount).toEqual(1);
        expect(serverApiServiceLoginStub.firstCall.args).toEqual([
          DummyAuthProvider.PROVIDER_ID,
          authState
        ]);
      });

      it("should emit LoggedInEvent to the server when login succeeded", async done => {
        const authEvent: AuthEventService = TestBed.get(AuthEventService);
        const user: Readonly<User> = {
          userId: "0123-4567-8901-2345",
          name: "Calvin"
        };
        const accessToken: Readonly<string> = "access-token";
        serverApiServiceLoginStub.returns(
          defer(() => Promise.resolve({ user, accessToken }))
        );

        await authService.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);

        authEvent.$event.subscribe(event => {
          expect(event).toEqual({
            type: EventType.LoggedIn,
            user,
            accessToken
          });

          done();
        });
      });

      it("should emit LogInFailedEvent to the server when login responds with error", async done => {
        const authEvent: AuthEventService = TestBed.get(AuthEventService);
        serverApiServiceLoginStub.returns(
          defer(() =>
            Promise.reject(new ServerApiError(ERROR_CODE.LOGIN_UNAUTHORIZED))
          )
        );

        await authService.tryAuthAndLogin(DummyAuthProvider.PROVIDER_ID);

        authEvent.$event.subscribe(event => {
          expect(event).toEqual({
            type: EventType.LogInFailed,
            errorCode: ERROR_CODE.LOGIN_UNAUTHORIZED
          });

          done();
        });
      });
    });
  });

  describe("logout()", () => {
    it("should emit user LoggedOutEvent", done => {
      const authService: AuthService = TestBed.get(AuthService);
      const authEvent: AuthEventService = TestBed.get(AuthEventService);

      authService.logout();

      authEvent.$event.subscribe(event => {
        expect(event).toEqual({
          type: EventType.LoggedOut,
          reason: LoggedOutReason.UserLogout
        });

        done();
      });
    });
  });

  describe("checkAuthState(); Given AuthStateInitialized event is emitted", () => {
    describe("When user is not logged in (AuthState access_token absent)", () => {
      beforeEach(() => {
        const fakeAuthState = {
          accessToken: null
        };
        TestBed.configureTestingModule({
          providers: [
            {
              provide: AuthStateService,
              useValue: fakeAuthState
            }
          ]
        });
      });

      describe("When AuthStorage last auth provider presents", () => {
        it("should emit AuthFailed when auth provider no longer exist", async done => {
          fakeAuthStorage.getItem.resetBehavior();
          fakeAuthStorage.getItem
            .withArgs("last_provider_id")
            .resolves(DummyAuthProvider.PROVIDER_ID);
          const authEvent = emitAuthStateInitialized();

          TestBed.get(AuthService);
          await nextEventLoop();

          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthFailed,
              providerId: DummyAuthProvider.PROVIDER_ID,
              errorCode: ERROR_CODE.AUTH_PROVIDER_NOT_FOUND
            });

            done();
          });
        });

        it("should emit AuthFailed when auth provider is not injectable", async done => {
          fakeAuthStorage.getItem.resetBehavior();
          fakeAuthStorage.getItem
            .withArgs("last_provider_id")
            .resolves(DummyAuthProvider.PROVIDER_ID);
          TestBed.configureTestingModule({
            providers: [
              {
                provide: AuthServiceConfig,
                useValue: new AuthServiceConfig([DummyAuthProvider])
              }
            ]
          });

          const authEvent = emitAuthStateInitialized();

          TestBed.get(AuthService);
          await nextEventLoop();

          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthFailed,
              providerId: DummyAuthProvider.PROVIDER_ID,
              errorCode: ERROR_CODE.AUTH_PROVIDER_NOT_INJECTABLE
            });

            done();
          });
        });

        it("should emit AuthFailed when auth provider fails verification", async done => {
          @Injectable()
          class AuthVerifyFailedProvider extends DummyAuthProvider {
            public async verify(): Promise<AuthResult> {
              return {
                status: AuthStatus.Failed,
                errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
              };
            }
          }

          const providerId = AuthVerifyFailedProvider.PROVIDER_ID;
          fakeAuthStorage.getItem.resetBehavior();
          fakeAuthStorage.getItem
            .withArgs("last_provider_id")
            .resolves(providerId);

          configureDummyAuthProviderInTestBed(AuthVerifyFailedProvider);

          const authEvent = emitAuthStateInitialized();

          TestBed.get(AuthService);
          await nextEventLoop();

          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthFailed,
              providerId,
              errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
            });

            done();
          });
        });

        it("should emit nothing when auth provider verify resolves to NotAuth", async done => {
          @Injectable()
          class AuthVerifyNotAuthProvider extends DummyAuthProvider {
            public async verify(): Promise<AuthResult> {
              return { status: AuthStatus.NoAuth };
            }
          }
          configureDummyAuthProviderInTestBed(AuthVerifyNotAuthProvider);

          TestBed.get(AuthService);
          const authEvent = emitAuthStateInitialized();
          await nextEventLoop();

          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthStateInitialized
            });

            done();
          });
        });

        it("should emit AuthSucceeded when auth provider verifies successfully", async done => {
          const authState = {
            authCode: "1234567890"
          };
          @Injectable()
          class AuthVerifySucceededProvider extends DummyAuthProvider {
            public async verify(): Promise<AuthResult> {
              return { status: AuthStatus.Succeeded, state: authState };
            }
          }
          configureDummyAuthProviderInTestBed(AuthVerifySucceededProvider);

          const providerId = AuthVerifySucceededProvider.PROVIDER_ID;
          fakeAuthStorage.getItem.resetBehavior();
          fakeAuthStorage.getItem
            .withArgs("last_provider_id")
            .resolves(providerId);

          TestBed.get(AuthService);
          const authEvent = emitAuthStateInitialized();
          await nextEventLoop();

          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthSucceeded,
              providerId,
              state: authState
            });

            done();
          });
        });
      });

      describe("When AuthStorage last auth provider absents", () => {
        it("should not emit any Auth event", async done => {
          fakeAuthStorage.getItem.resetBehavior();
          fakeAuthStorage.getItem.withArgs("last_provider_id").resolves(null);

          TestBed.get(AuthService);
          const authEvent = emitAuthStateInitialized();
          await nextEventLoop();

          authEvent.$event.subscribe(event => {
            expect(event).toEqual({
              type: EventType.AuthStateInitialized
            });

            done();
          });
        });
      });
    });

    describe("When user is logged in (AuthState access_token presents)", () => {
      const prevAccessToken: Readonly<string> = "prev-access-token";
      let fakeAuthState: {
        accessToken: string;
      };
      let serverApiServiceGetLoginStatusStub: sinon.SinonStub;
      beforeEach(() => {
        fakeAuthState = {
          accessToken: prevAccessToken
        };
        serverApiServiceGetLoginStatusStub = sinon.stub();
        TestBed.configureTestingModule({
          providers: [
            {
              provide: AuthStateService,
              useValue: fakeAuthState
            },
            {
              provide: ServerApiService,
              useValue: {
                getLoginStatus: serverApiServiceGetLoginStatusStub
              }
            }
          ]
        });
      });

      it("should emit LoggedOut event when access token no longer valid", async done => {
        const respondUnauthorized = defer(() =>
          Promise.reject(
            new ServerApiError<LOGIN_ERROR_CODE>(ERROR_CODE.LOGIN_UNAUTHORIZED)
          )
        );
        serverApiServiceGetLoginStatusStub.returns(respondUnauthorized);
        const authEvent: AuthEventService = TestBed.get(AuthEventService);

        TestBed.get(AuthService);
        emitAuthStateInitialized();
        await nextEventLoop();
        await nextEventLoop();

        authEvent.$event.subscribe(event => {
          expect(event).toEqual({
            type: EventType.LoggedOut,
            reason: LoggedOutReason.SessionExpired
          });

          done();
        });
      });

      it("should emit LoggedIn event when access token is still valid", async done => {
        const user: Readonly<User> = {
          userId: "0123-4567-8901-2345",
          name: "Calvin"
        };
        const accessToken = "access-token";
        const respondUnauthorized = defer(() =>
          Promise.resolve({
            user,
            accessToken: accessToken
          })
        );
        serverApiServiceGetLoginStatusStub.returns(respondUnauthorized);
        const authEvent: AuthEventService = TestBed.get(AuthEventService);

        TestBed.get(AuthService);
        emitAuthStateInitialized();
        await nextEventLoop();
        await nextEventLoop();

        authEvent.$event.subscribe(event => {
          expect(event).toEqual({
            type: EventType.LoggedIn,
            user,
            accessToken
          });

          done();
        });
      });
    });

    const emitAuthStateInitialized = (): AuthEventService => {
      const authEvent: AuthEventService = TestBed.get(AuthEventService);
      authEvent.authStateInitialized();

      return authEvent;
    };
  });
});

describe("AuthServiceConfig", () => {
  it("should construct a mapping of PROVIDER_ID to AuthProvider class", () => {
    class FooProvider extends DummyAuthProvider {
      public static PROVIDER_ID = "Foo";
    }
    class BarProvider extends DummyAuthProvider {
      public static PROVIDER_ID = "Bar";
    }

    const config = new AuthServiceConfig([FooProvider, BarProvider]);

    expect(config.providers).toEqual({
      Foo: FooProvider,
      Bar: BarProvider
    });
  });
});

const configureDummyAuthProviderInTestBed = (
  authProvider: Class<DummyAuthProvider>
) => {
  TestBed.configureTestingModule({
    providers: [
      {
        provide: AuthServiceConfig,
        useValue: new AuthServiceConfig([authProvider])
      },
      authProvider,
      {
        provide: DummyInjectable,
        useValue: new DummyInjectable()
      }
    ]
  });
};
