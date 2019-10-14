import * as uuidv4 from "uuid/v4";
import { ERROR_CODE } from "../../../types";
import { AuthResult, AuthProvider, AuthStatus } from "./auth-provider";
import { AuthStorageService } from "../auth-storage.service";
import { Injectable } from "@angular/core";

@Injectable()
export abstract class OAuthProvider extends AuthProvider {
  private readonly AUTH_STORAGE_STATE_KEY = "auth_last_state";
  private readonly AUTH_CALLBACK_PATH = "/auth/callback";

  constructor(private authStorage: AuthStorageService) {
    super();
  }

  public async tryAuth(): Promise<AuthResult> {
    const state = this.generateState();
    await this.storeStateToStorage(state);

    const authUrl = new URL(this.getEndpoint());
    authUrl.searchParams.append("client_id", this.getClientId());
    authUrl.searchParams.append("prompt", "select_account");
    authUrl.searchParams.append("redirect_uri", this.getRedirectUri());
    authUrl.searchParams.append("response_type", "code");
    authUrl.searchParams.append("scope", this.getEmailScope());
    authUrl.searchParams.append("state", state);

    window.location.assign(authUrl.toString());

    return {
      status: AuthStatus.Postponed
    };
  }

  private generateState(): string {
    return uuidv4();
  }

  private storeStateToStorage(state: string): Promise<void> {
    return this.authStorage.setItem(this.AUTH_STORAGE_STATE_KEY, state);
  }

  /**
   * Get OAuth provider endpoint URL
   */
  protected abstract getEndpoint(): string;
  /**
   * Get application ClientID assigned by the OAuth provider
   */
  protected abstract getClientId(): string;
  /**
   * Get the necessary scope(s) to obtain user email resource
   */
  protected abstract getEmailScope(): string;

  private getRedirectUri(): string {
    return `${this.getHost()}${this.AUTH_CALLBACK_PATH}`;
  }

  private getHost(): string {
    console.log(61, `${window.location.protocol}//${window.location.host}`);
    return `${window.location.protocol}//${window.location.host}`;
  }

  public async verify(): Promise<AuthResult> {
    const callbackUrl = new URL(window.location.href);
    if (callbackUrl.pathname !== this.AUTH_CALLBACK_PATH) {
      return {
        status: AuthStatus.NoAuth
      };
    }

    const storedState = await this.getLastState();
    console.log(74, storedState);
    if (!storedState) {
      return {
        status: AuthStatus.NoAuth
      };
    }

    const callbackUrlParams = callbackUrl.searchParams;
    console.log(82, callbackUrlParams);
    console.log(83, callbackUrlParams.get("error"));
    if (callbackUrlParams.get("error")) {
      return {
        status: AuthStatus.Failed,
        errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
      };
    }

    console.log(91, callbackUrlParams.get("state"));
    if (callbackUrlParams.get("state") !== storedState) {
      return {
        status: AuthStatus.Failed,
        errorCode: ERROR_CODE.AUTH_STATE_MISMATCH
      };
    }

    const authCode = callbackUrlParams.get("code");
    console.log(100, callbackUrlParams.get("code"));
    if (!authCode) {
      return {
        status: AuthStatus.Failed,
        errorCode: ERROR_CODE.AUTH_UNAUTHORIZED
      };
    }

    this.authStorage
      .removeItem(this.AUTH_STORAGE_STATE_KEY)
      .catch(console.error);
    console.log(111, {
      status: AuthStatus.Succeeded,
      state: {
        authCode
      }
    });
    return {
      status: AuthStatus.Succeeded,
      state: {
        authCode
      }
    };
  }

  private async getLastState(): Promise<string> {
    return this.authStorage.getItem(this.AUTH_STORAGE_STATE_KEY);
  }
}
