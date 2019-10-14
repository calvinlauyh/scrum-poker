import { Injectable } from '@angular/core';
import { OAuthProvider } from './oauth-provider';
import { AuthStorageService } from '../auth-storage.service';

@Injectable()
export class GoogleAuthProvider extends OAuthProvider{
  public static PROVIDER_ID: Readonly<string> = "GOOGLE";

  constructor(authStorage: AuthStorageService) {
    super(authStorage);
  }

  /**
   * Get OAuth provider endpoint URL
   */
  protected getEndpoint(): string {
    return "https://accounts.google.com/o/oauth2/v2/auth";
  }
  /**
   * Get application ClientID assigned by the OAuth provider
   */
  protected getClientId(): string {
    return "565603376938-rueku0o99h2c4tmbr6287br0ginsf8tf.apps.googleusercontent.com";
  }
  /**
   * Get the necessary scope(s) to obtain user email resource
   */
  protected getEmailScope(): string {
    return "email";
  }
}
