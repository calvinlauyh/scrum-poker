import { Injectable } from "@angular/core";
import { HttpClient, HttpHeaders } from "@angular/common/http";
import { Observable, throwError } from "rxjs";
import { catchError } from "rxjs/operators";
import { ERROR_CODE, User, LOGIN_ERROR_CODE } from "../../types";

@Injectable({
    providedIn: "root"
})
export class ServerApiService {
    private readonly BASE_API_URL = "api";

    constructor(private http: HttpClient) {}

    public login(
        authProviderId: string,
        state: any
    ): Observable<LoginResponse> {
        const url = `${this.BASE_API_URL}/login`;
        const request: LoginRequest = {
            authProviderId,
            state
        };
        return this.http.post<LoginResponse>(url, request).pipe(
            catchError(err => {
                switch (err.status) {
                    case 401:
                        return throwError(
                            new ServerApiError<LOGIN_ERROR_CODE>(
                                ERROR_CODE.LOGIN_UNAUTHORIZED
                            )
                        );
                    default:
                        return throwError(
                            new ServerApiError<LOGIN_ERROR_CODE>(
                                ERROR_CODE.UNKNOWN_ERROR
                            )
                        );
                }
            })
        );
    }

    public getLoginStatus(accessToken: string): Observable<LoginStatusResponse> {
        const url = `${this.BASE_API_URL}/login/status`;
        return this.http.get<LoginResponse>(url, {
            headers: this.generateHeader(accessToken),
        }).pipe(
            catchError(err => {
                switch (err.status) {
                    case 401:
                        return throwError(
                            new ServerApiError<LOGIN_ERROR_CODE>(
                                ERROR_CODE.LOGIN_UNAUTHORIZED
                            )
                        );
                    default:
                        return throwError(
                            new ServerApiError<LOGIN_ERROR_CODE>(
                                ERROR_CODE.UNKNOWN_ERROR
                            )
                        );
                }
            })
        );
    }

    private generateHeader = (accessToken: string): HttpHeaders => {
        return new HttpHeaders({
            "X-Api-Token": accessToken
        });
    }
}

interface LoginRequest {
    authProviderId: string;
    state: any;
}
interface LoginResponse {
    user: User;
    accessToken: string;
}

interface LoginStatusResponse {
    user: User;
    accessToken: string;
}

export class ServerApiError<E extends ERROR_CODE> extends Error {
    constructor(public errorCode: E) {
        super(errorCode);
    }
}
