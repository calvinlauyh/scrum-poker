import { Injectable, Injector } from "@angular/core";
import { Class, ERROR_CODE, LOGIN_ERROR_CODE } from "../../types";
import {
  AuthProvider,
  AuthFailedResult,
  AuthSucceededResult,
  AuthResult,
  AuthStatus
} from "./provider/auth-provider";
import { ServerApiService, ServerApiError } from "../server/server-api.service";
import { AuthStateService } from "./auth-state.service";
import { AuthStorageService } from "./auth-storage.service";
import {
  AuthEventService,
  Event,
  EventType,
  LoggedOutReason
} from "./auth-event.service";
import { OAuthProvider } from './provider/oauth-provider';

export class AuthServiceConfig {
  public providers: AuthProviderMapping = {};

  constructor(providers: Class<AuthProvider>[]) {
    for (const provider of providers) {
      this.providers[provider["PROVIDER_ID"]] = provider;
    }
  }
}

@Injectable({
  providedIn: "root"
})
export class AuthService {
  private providers: AuthProviderMapping;

  private readonly AUTH_STORAGE_LAST_PROVIDER_ID_KEY = "last_provider_id";

  constructor(
    config: AuthServiceConfig,
    private injector: Injector,
    private authEvent: AuthEventService,
    private authState: AuthStateService,
    private authStorage: AuthStorageService,
    private serverApi: ServerApiService
  ) {
    this.providers = config.providers;

    this.listenToAuthEvent();
  }

  private listenToAuthEvent() {
    this.authEvent.$event.subscribe(event => this.handleAuthEvent(event));
  }

  private handleAuthEvent(event: Event) {
    switch (event.type) {
      case EventType.AuthStateInitialized:
        this.checkAuthState();
        break;
      case EventType.AuthFailed:
        this.resetLastProviderId().catch(console.error);
        break;
      case EventType.AuthSucceeded:
        this.resetLastProviderId().catch(console.error);
        this.login(event.providerId, event.state);
        break;
    }
  }

  private async checkAuthState(): Promise<void> {
    const accessToken = await this.authState.accessToken;
    if (accessToken) {
      await this.verifyLoginStatus(accessToken);
      return;
    }

    const lastProviderId = await this.authStorage.getItem(
      this.AUTH_STORAGE_LAST_PROVIDER_ID_KEY
    );
    if (lastProviderId) {
      await this.verifyLastAuthAttempt(lastProviderId);
    }
  }

  private async verifyLoginStatus(accessToken: string): Promise<void> {
    this.serverApi.getLoginStatus(accessToken).subscribe(
      response => {
        this.authEvent.loggedIn(response.user, response.accessToken);
      },
      err => {
        this.authEvent.loggedOut(LoggedOutReason.SessionExpired);
      }
    );
  }

  private async verifyLastAuthAttempt(providerId: string): Promise<void> {
    try {
      const provider = this.getProviderById(providerId);

      const authResult = await provider.verify();
      this.handleAuthResult(authResult, providerId);
    } catch (err) {
      console.error(err);

      this.handleAuthError(err, providerId);
    }
  }

  /**
   * Try to authenticate the user with the authentication provider, and try to
   * login with auth result when succeeded
   */
  public async tryAuthAndLogin(providerId: string) {
    try {
      const provider = this.getProviderById(providerId);

      await this.updateLastProviderId(providerId);

      const authResult = await provider.tryAuth();
      this.handleAuthResult(authResult, providerId);
    } catch (err) {
      this.handleAuthError(err, providerId);

      throw err;
    }
  }

  private getProviderById = (providerId: string): AuthProvider => {
    if (!this.providers[providerId]) {
      this.authEvent.authFailed(providerId, ERROR_CODE.AUTH_PROVIDER_NOT_FOUND);
      throw new ProviderNotFoundError(providerId);
    }

    try {
      return this.injectAndGetProviderById(providerId);
    } catch (err) {
      this.authEvent.authFailed(
        providerId,
        ERROR_CODE.AUTH_PROVIDER_NOT_INJECTABLE
      );
      throw new ProviderNotInjectableError(providerId, err);
    }
  };

  private injectAndGetProviderById = (providerId: string): AuthProvider => {
    const provider = this.providers[providerId];
    return this.injector.get<AuthProvider>(provider);
  };

  private updateLastProviderId = async (providerId: string) => {
    await this.authStorage.setItem(
      this.AUTH_STORAGE_LAST_PROVIDER_ID_KEY,
      providerId
    );
  };

  private handleAuthError(err: Error, providerId): void {
    if (err instanceof ProviderNotFoundError) {
      this.authEvent.authFailed(providerId, ERROR_CODE.AUTH_PROVIDER_NOT_FOUND);
    } else if (err instanceof ProviderNotInjectableError) {
      this.authEvent.authFailed(
        providerId,
        ERROR_CODE.AUTH_PROVIDER_NOT_INJECTABLE
      );
    } else {
      this.authEvent.authFailed(providerId, ERROR_CODE.UNKNOWN_ERROR);
    }
  }

  private handleAuthResult(authResult: AuthResult, providerId: string): void {
    console.log(172, authResult);
    if (authResult.status === AuthStatus.Succeeded) {
      this.authEvent.authSucceeded(
        providerId,
        (authResult as AuthSucceededResult).state
      );
    } else if (authResult.status === AuthStatus.Failed) {
      this.authEvent.authFailed(
        providerId,
        (authResult as AuthFailedResult).errorCode
      );
      return;
    }
  }

  public logout() {
    // TODO: Call logout API
    this.authEvent.loggedOut(LoggedOutReason.UserLogout);
  }

  private async resetLastProviderId(): Promise<void> {
    return this.authStorage.removeItem(this.AUTH_STORAGE_LAST_PROVIDER_ID_KEY);
  }

  private login(authProviderId: string, state: any): void {
    this.serverApi
      .login(authProviderId, state)
      .subscribe(
        loginResponse =>
          this.authEvent.loggedIn(
            loginResponse.user,
            loginResponse.accessToken
          ),
        (err: ServerApiError<LOGIN_ERROR_CODE>) =>
          this.authEvent.logInFailed(err.errorCode)
      );
  }
}

interface AuthProviderMapping {
  [providerId: string]: Class<AuthProvider>;
}

export class ProviderNotFoundError extends Error {
  constructor(providerId: string) {
    super(`Provider "${providerId}" not found`);
  }
}

export class ProviderNotInjectableError extends Error {
  public originalError: Error;
  constructor(providerId: string, err: Error) {
    super(`Provider "${providerId}" not injectable: ${err.toString()}`);
    this.originalError = err;
  }
}
