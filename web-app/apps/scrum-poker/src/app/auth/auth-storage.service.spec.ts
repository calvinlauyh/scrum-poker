import { TestBed } from '@angular/core/testing';
import * as sinon from 'sinon';

import { AuthStorageService } from './auth-storage.service';

describe('AuthStorageService', () => {
  const window: Readonly<any> = {
    localStorage: {
      setItem: () => {},
      getItem: () => {},
      removeItem: () => {}
    }
  };
  let sandbox: sinon.SinonSandbox;
  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        {
          provide: Window,
          useValue: window
        }
      ]
    });
    sandbox = sinon.createSandbox();
  });

  afterEach(() => {
    sandbox.restore();
  });

  it('should be created', () => {
    const service: AuthStorageService = TestBed.get(AuthStorageService);

    expect(service).toBeTruthy();
  });

  describe('setItem', () => {
    it('should set item in local storage with prefix', async () => {
      const setItemStub = sandbox.stub(window.localStorage, 'setItem');

      const service: AuthStorageService = TestBed.get(AuthStorageService);

      const key = 'user';
      const value = '{"user_id":0,"name":"Calvin"}';
      await service.setItem(key, value);

      expect(setItemStub.callCount).toEqual(1);
      expect(setItemStub.firstCall.args).toEqual([`auth_${key}`, value]);
    });
  });

  describe('removeItem', () => {
    it('should remove item in local storage with prefix', async () => {
      const removeItemStub = sandbox.stub(window.localStorage, 'removeItem');

      const service: AuthStorageService = TestBed.get(AuthStorageService);

      const key = 'user';
      await service.removeItem(key);

      expect(removeItemStub.callCount).toEqual(1);
      expect(removeItemStub.firstCall.args).toEqual([`auth_${key}`]);
    });
  });


  describe('getItem', () => {
    it('should get item from local storage of the key with prefix', async () => {
      const key = 'user';
      const value = '{"user_id":0,"name":"Calvin"}';
      sandbox
        .stub(window.localStorage, 'getItem')
        .withArgs(`auth_${key}`)
        .returns(value);

      const service: AuthStorageService = TestBed.get(AuthStorageService);

      return expect(service.getItem(key)).resolves.toEqual(value);
    });
  });
});
