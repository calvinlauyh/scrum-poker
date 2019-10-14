import { AUTH_ERROR_CODE } from "../../../types";

export abstract class AuthProvider {
  /**
   * Try to authenticate user from the provider service. Returns AuthResult
   * based on authentication result.
   */
  public abstract tryAuth(): Promise<AuthResult>;
  /**
   * Verify if last authentication has been successful based on current
   * state. Returns AuthResult based on the current state.
   */
  public abstract verify(): Promise<AuthResult>;
}

export type AuthResult =
  | AuthPostponedResult
  | AuthNotAuthResult
  | AuthSucceededResult
  | AuthFailedResult;
interface BaseAuthResult {
  status: AuthStatus;
}
export interface AuthPostponedResult extends BaseAuthResult {
  status: AuthStatus.Postponed;
}
export interface AuthNotAuthResult extends BaseAuthResult {
  status: AuthStatus.NoAuth;
}
export interface AuthSucceededResult extends BaseAuthResult {
  status: AuthStatus.Succeeded;
  state: any;
}
export interface AuthFailedResult extends BaseAuthResult {
  status: AuthStatus.Failed;
  errorCode: AUTH_ERROR_CODE;
}
export enum AuthStatus {
  Postponed = "Postponed",
  NoAuth = "NoAuth",
  Succeeded = "Succeeded",
  Failed = "Failed"
}
