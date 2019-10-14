import { TestBed } from "@angular/core/testing";
import { Injectable } from "@angular/core";
import * as sinon from "sinon";
import * as jsdom from "jsdom";
import { URL } from "url";

import { AuthStorageService } from "../auth-storage.service";
import { OAuthProvider } from "./oauth-provider";
import { AuthStatus } from "./auth-provider";
import { ERROR_CODE } from "../../../types";
import { nextEventLoop } from '../../utils';

describe("OAuthProvider", () => {
  let savedWindowLocation: Location;
  let sandbox: sinon.SinonSandbox;
  let fakeAuthStorage: {
    getItem: sinon.SinonStub;
    setItem: sinon.SinonStub;
    removeItem: sinon.SinonStub;
    clear: sinon.SinonStub;
  };

  const oAuthEndpoint: Readonly<string> = "https://localhost/oauth/";
  const clientId: Readonly<string> = "client-id";
  const emailScope: Readonly<string> = "email";

  @Injectable()
  class SimpleOAuthProvider extends OAuthProvider {
    protected getEndpoint(): string {
      return oAuthEndpoint;
    }
    protected getClientId(): string {
      return clientId;
    }
    protected getEmailScope(): string {
      return emailScope;
    }
  }

  beforeAll(() => {
    savedWindowLocation = (global as any).window.location;
  });

  beforeEach(() => {
    sandbox = sinon.createSandbox();

    TestBed.configureTestingModule({
      providers: [SimpleOAuthProvider]
    });

    fakeAuthStorage = {
      getItem: sinon.stub().rejects(new Error("getItem not implemented")),
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

    sandbox.stub(console, "error");
  });

  afterEach(() => {
    sandbox.restore();
  });

  afterAll(() => {
    (global as any).window.location = savedWindowLocation;
  });

  describe("tryAuth()", () => {
    const hostname: Readonly<string> = "https://localhost:8080/";

    let locationAssignStub: sinon.SinonStub;

    beforeEach(() => {
      delete (global as any).window;
      (global as any).window = new jsdom.JSDOM(undefined, {
        url: hostname
      }).window;
      locationAssignStub = sandbox.stub(
        (global as any).window.location,
        "assign"
      );
    });

    it("should resolve to auth postponed result", () => {
      const provider = TestBed.get(SimpleOAuthProvider);

      return expect(provider.tryAuth()).resolves.toEqual({
        status: AuthStatus.Postponed
      });
    });

    it("should redirect to OAuth endpoint with correct parameters", async () => {
      const provider = TestBed.get(SimpleOAuthProvider);

      await provider.tryAuth();

      expect(locationAssignStub.callCount).toEqual(1);
      expect(
        locationAssignStub.firstCall.calledWith(sinon.match.string)
      ).toBeTruthy();
      const oAuthUri = locationAssignStub.firstCall.args[0];
      const oAuthUrl = new URL(oAuthUri);

      expect(
        `${oAuthUrl.protocol}//${oAuthUrl.host}${oAuthUrl.pathname}`
      ).toEqual(oAuthEndpoint);
      const oAuthUrlParams = oAuthUrl.searchParams;
      expect(oAuthUrlParams.get("client_id")).toEqual(clientId);
      expect(oAuthUrlParams.get("redirect_uri")).toEqual(
        `${hostname}auth/callback`
      );
      expect(oAuthUrlParams.get("response_type")).toEqual("code");
      expect(oAuthUrlParams.get("scope")).toEqual(emailScope);
    });

    it("should generate unique state and store into AuthStorage", async () => {
      const provider = TestBed.get(SimpleOAuthProvider);

      await provider.tryAuth();

      const oAuthUri = locationAssignStub.firstCall.args[0];
      const oAuthUrl = new URL(oAuthUri);

      expect(fakeAuthStorage.setItem.callCount).toEqual(1);
      expect(
        fakeAuthStorage.setItem.firstCall.calledWith(
          "auth_last_state",
          sinon.match.string
        )
      ).toBeTruthy();
      const state = fakeAuthStorage.setItem.firstCall.args[1];

      const oAuthUrlParams = oAuthUrl.searchParams;
      expect(oAuthUrlParams.get("state")).toEqual(state);
    });
  });

  describe("verify()", () => {
    it("should resolves to not auth result when it is not callback URL", () => {
      delete (global as any).window;
      (global as any).window = new jsdom.JSDOM(undefined, {
        url: "https://localhost/login"
      }).window;

      const provider = TestBed.get(SimpleOAuthProvider);

      return expect(provider.verify()).resolves.toEqual({
        status: AuthStatus.NoAuth
      });
    });

    it("should resolves to auth failed result when OAuth callback with error", async () => {
      const redirectUri = "https://localhost/auth/callback";
      const state = "state";
      fakeAuthStorage.getItem.resetBehavior();
      fakeAuthStorage.getItem.withArgs("auth_last_state").resolves(state);

      delete (global as any).window;
      (global as any).window = new jsdom.JSDOM(undefined, {
        url: `${redirectUri}?error=access_denied`
      }).window;

      const provider = TestBed.get(SimpleOAuthProvider);

      return expect(provider.verify()).resolves.toEqual({
        status: AuthStatus.Failed,
        errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
      });
    });

    it("should resolves to not auth result when AuthStorage does not have last state", async () => {
      const redirectUri = "https://localhost/auth/callback";
      const callbackState = "callback-state";
      fakeAuthStorage.getItem.resetBehavior();
      fakeAuthStorage.getItem.withArgs("auth_last_state").resolves(null);

      delete (global as any).window;
      (global as any).window = new jsdom.JSDOM(undefined, {
        url: `${redirectUri}?code=4/P7q7W91a-oMsCeLvIaQm6bTrgtp7&state=${callbackState}&client_id=${clientId}&client_secret=client_secret&redirect_uri=${redirectUri}&grant_type=authorization_code`
      }).window;

      const provider = TestBed.get(SimpleOAuthProvider);

      return expect(provider.verify()).resolves.toEqual({
        status: AuthStatus.NoAuth
      });
    });

    it("should resolves to auth failed result when OAuth callback with different state than AuthStorage last state", async () => {
      const redirectUri = "https://localhost/auth/callback";
      const authStorageState = "auth-storage-state";
      const callbackState = "callback-state";
      fakeAuthStorage.getItem.resetBehavior();
      fakeAuthStorage.getItem
        .withArgs("auth_last_state")
        .resolves(authStorageState);

      delete (global as any).window;
      (global as any).window = new jsdom.JSDOM(undefined, {
        url: `${redirectUri}?code=4/P7q7W91a-oMsCeLvIaQm6bTrgtp7&state=${callbackState}&client_id=${clientId}&client_secret=client_secret&redirect_uri=${redirectUri}&grant_type=authorization_code`
      }).window;

      const provider = TestBed.get(SimpleOAuthProvider);

      return expect(provider.verify()).resolves.toEqual({
        status: AuthStatus.Failed,
        errorCode: ERROR_CODE.AUTH_STATE_MISMATCH
      });
    });

    it("should resolves to auth failed result when OAuth callback without code", async () => {
      const redirectUri = "https://localhost/auth/callback";
      const state = "state";
      fakeAuthStorage.getItem.resetBehavior();
      fakeAuthStorage.getItem
        .withArgs("auth_last_state")
        .resolves(state);

      delete (global as any).window;
      (global as any).window = new jsdom.JSDOM(undefined, {
        url: `${redirectUri}?state=${state}&client_id=${clientId}&client_secret=client_secret&redirect_uri=${redirectUri}&grant_type=authorization_code`
      }).window;

      const provider = TestBed.get(SimpleOAuthProvider);

      return expect(provider.verify()).resolves.toEqual({
        status: AuthStatus.Failed,
        errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
      });
    });

    describe("When OAuth callback with code", () => {
      const redirectUri: Readonly<string> = "https://localhost/auth/callback";
      const state: Readonly<string> = "state";
      const authCode: Readonly<string> = "4/P7q7W91a-oMsCeLvIaQm6bTrgtp7";
      beforeEach(() => {
        fakeAuthStorage.getItem.resetBehavior();
        fakeAuthStorage.getItem.withArgs("auth_last_state").resolves(state);

        delete (global as any).window;
        (global as any).window = new jsdom.JSDOM(undefined, {
          url: `${redirectUri}?code=${authCode}&client_id=${clientId}&state=${state}&client_secret=client_secret&redirect_uri=${redirectUri}&grant_type=authorization_code`
        }).window;
      });

      it("should resolves to auth succeeded result with auth code when OAuth callback successfully", () => {
        const provider = TestBed.get(SimpleOAuthProvider);

        return expect(provider.verify()).resolves.toEqual({
          status: AuthStatus.Succeeded,
          state: {
            authCode
          }
        });
      });

      it("should try to clear AuthStorage last state", async () => {
        const provider = TestBed.get(SimpleOAuthProvider);

        await provider.verify();
        await nextEventLoop();

        expect(fakeAuthStorage.removeItem.callCount).toEqual(1);
        expect(fakeAuthStorage.removeItem.calledWith("auth_last_state")).toBeTruthy();
      });

      it("should resolves to auth succeeded result regardless of clear AuthStorage error", async () => {
        fakeAuthStorage.removeItem.resetBehavior();
        fakeAuthStorage.removeItem.rejects(new Error("Storage error"));

        const provider = TestBed.get(SimpleOAuthProvider);

        await expect(provider.verify()).resolves.toEqual({
          status: AuthStatus.Succeeded,
          state: {
            authCode
          }
        });
        expect(fakeAuthStorage.removeItem.callCount).toEqual(1);
        expect(fakeAuthStorage.removeItem.calledWith("auth_last_state")).toBeTruthy();
      });
    });
  });
});
