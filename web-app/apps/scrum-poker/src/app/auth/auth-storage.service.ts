import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class AuthStorageService {
  private readonly KEY_PREFIX = 'auth_';

  constructor(private window: Window) { }

  public async getItem(key: string): Promise<string> {
    return this.window.localStorage.getItem(this.getKey(key));
  }

  public async removeItem(key: string): Promise<void> {
    this.window.localStorage.removeItem(this.getKey(key));
  }

  public async setItem(key: string, value: string): Promise<void> {
    this.window.localStorage.setItem(this.getKey(key), value);
  }

  public async clear(): Promise<void> {
    this.window.localStorage.clear();
  }

  private getKey(key: string) {
    return `${this.KEY_PREFIX}${key}`;
  }
}
