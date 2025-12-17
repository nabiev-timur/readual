# [readual] Readual Project

Это основной проект для работы с текстом и Markdown файлами.

## Testing

### fs-isolation

```bash
ls ~
```

## [env] Окружение

### Linux-x64-rpm

```bash
echo readual::env::Linux-x64-rpm
```

### Linux-x86-rpm

```bash
echo readual::env::Linux-x86-rpm
```

### Linux-x64-deb

```bash
echo readual::env::Linux-x64-deb
```

### Linux-x86-deb

```bash
echo readual::env::Linux-x86-deb
```

### Windows

Описание как установить msvc

## [build] Сборка

### Linux
<!-- rdl:alias=linux -->

#### x64

##### Release
<!-- rdl:alias=release -->

`echo readual::build::Linux::x64::Release`

##### Debug
<!-- rdl:alias=debug -->

`echo readual::build::Linux::x64::Release`

#### x86

##### Release
<!-- rdl:alias=release -->

`echo readual::build::Linux::x86::Release`

##### Debug
<!-- rdl:alias=debug -->

`echo readual::build::Linux::x86::Debug`

### Windows

#### x64

##### Release
<!-- rdl:alias=release -->

`echo readual::build::Windows::x64::Release`

##### Debug
<!-- rdl:alias=debug -->

`echo readual::build::Windows::x64::Debug`

#### x86

##### Release
<!-- rdl:alias=release -->

`echo readual::build::Windows::x86::Release`

##### Debug
<!-- rdl:alias=debug -->

`echo readual::build::Windows::x86::Debug`

## [test] Тестирование

### Linux
<!-- rdl:alias=linux -->

#### x64

##### Release
<!-- rdl:alias=release -->
<!-- rdl:deps=build::Linux::x64::release -->

`echo readual::test::Linux::x64::Release`

##### Debug
<!-- rdl:alias=debug -->
<!-- rdl:deps=build::Linux::x64::debug -->

`echo readual::test::Linux::x64::Debug`

#### x86

##### Release
<!-- rdl:alias=release -->
<!-- rdl:deps=build::Linux::x86::release -->

`echo readual::test::Linux::x86::Release`

##### Debug
<!-- rdl:alias=debug -->
<!-- rdl:deps=build::Linux::x86::release -->

`echo readual::test::Linux::x86::Debug`

### Windows

#### x64

##### Release
<!-- rdl:alias=release -->
<!-- rdl:deps=build::Windows::x64::release -->

`echo readual::test::Windows::x64::Release`

##### Debug
<!-- rdl:alias=debug -->
<!-- rdl:deps=build::Windows::x64::debug -->

`echo readual::test::Windows::x64::Debug`

#### x86

##### Release
<!-- rdl:alias=release -->
<!-- rdl:deps=build::Windows::x86::release -->

`echo readual::test::Windows::x86::Release`

##### Debug
<!-- rdl:alias=debug -->
<!-- rdl:deps=build::Windows::x86::debug -->

`echo readual::test::Windows::x86::Debug`

## fs-test

```bash
ls ~/.ssh
```