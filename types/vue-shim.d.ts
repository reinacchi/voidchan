export interface Captcha {
  uuid: string
  svg: string
}

export interface Submission {
  captcha: string
  uuid: string
}
